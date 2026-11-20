//! Модульные тесты для `background_service.rs`.
//!
//! Подключаются в `background_service.rs` строками
//! `#[cfg(test)] #[path = "background_service.tests.rs"] mod tests;`.

use std::sync::Mutex;

use crate::ports::SettingsRepository;

use super::*;

/// Начало PNG-файла (сигнатура + немного данных).
const PNG: [u8; 9] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1];
/// Начало JPEG-файла.
const JPEG: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xE0];

/// Хранилище фонов в памяти.
#[derive(Default)]
struct MemoryStore(Mutex<Vec<(BackgroundImage, Vec<u8>)>>);

impl BackgroundStore for MemoryStore {
    fn list(&self) -> Result<Vec<BackgroundImage>, RepositoryError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|(image, _)| image.clone())
            .collect())
    }

    fn load(
        &self,
        id: &BackgroundId,
    ) -> Result<Option<(BackgroundImage, Vec<u8>)>, RepositoryError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .find(|(image, _)| &image.id == id)
            .cloned())
    }

    fn save(&self, image: &BackgroundImage, bytes: &[u8]) -> Result<(), RepositoryError> {
        let mut images = self.0.lock().unwrap();
        if !images.iter().any(|(saved, _)| saved.id == image.id) {
            images.push((image.clone(), bytes.to_vec()));
        }
        Ok(())
    }

    fn remove(&self, id: &BackgroundId) -> Result<(), RepositoryError> {
        self.0.lock().unwrap().retain(|(image, _)| &image.id != id);
        Ok(())
    }
}

/// Настройки в памяти.
#[derive(Default)]
struct MemorySettings(Mutex<Option<AppSettings>>);

impl SettingsRepository for MemorySettings {
    fn load(&self) -> Result<Option<AppSettings>, RepositoryError> {
        Ok(self.0.lock().unwrap().clone())
    }

    fn save(&self, settings: &AppSettings) -> Result<(), RepositoryError> {
        *self.0.lock().unwrap() = Some(settings.clone());
        Ok(())
    }
}

/// Сервис с пустыми хранилищами; второй элемент — настройки (чтобы выбрать фон в тесте).
fn service() -> (BackgroundService, Arc<SettingsService>) {
    let settings = Arc::new(SettingsService::new(Arc::new(MemorySettings::default())));
    let service = BackgroundService::new(Arc::new(MemoryStore::default()), Arc::clone(&settings));
    (service, settings)
}

/// Выбирает фон в настройках.
fn select(settings: &SettingsService, id: &BackgroundId) {
    settings
        .update_settings(AppSettings {
            background: Some(id.clone()),
            ..AppSettings::default()
        })
        .unwrap();
}

#[test]
fn added_image_is_listed_and_readable() {
    let (service, _) = service();

    let image = service.add(&PNG).unwrap();

    assert_eq!(image.format, ImageFormat::Png);
    assert_eq!(image.id, BackgroundId::from_content(&PNG));
    assert_eq!(service.list().unwrap(), vec![image.clone()]);
    assert_eq!(
        service.image(&image.id).unwrap(),
        Some((ImageFormat::Png, PNG.to_vec()))
    );
}

#[test]
fn same_file_is_stored_once() {
    let (service, _) = service();

    let first = service.add(&JPEG).unwrap();
    let second = service.add(&JPEG).unwrap();

    assert_eq!(first, second);
    assert_eq!(service.list().unwrap().len(), 1);
}

#[test]
fn unsupported_file_is_rejected() {
    let (service, _) = service();

    assert!(matches!(
        service.add(b"GIF89a"),
        Err(BackgroundError::UnsupportedFormat)
    ));
    assert!(service.list().unwrap().is_empty());
}

#[test]
fn too_large_file_is_rejected() {
    let (service, _) = service();
    let too_large = usize::try_from(MAX_BACKGROUND_BYTES).unwrap() + 1;
    let mut bytes = PNG.to_vec();
    bytes.resize(too_large, 0);

    assert!(matches!(
        service.add(&bytes),
        Err(BackgroundError::TooLarge)
    ));
}

#[test]
fn missing_image_is_none() {
    let (service, _) = service();

    assert_eq!(
        service.image(&BackgroundId::parse("abc").unwrap()).unwrap(),
        None
    );
}

#[test]
fn removing_selected_image_clears_selection() {
    let (service, settings) = service();
    let image = service.add(&PNG).unwrap();
    select(&settings, &image.id);

    let after = service.remove(&image.id).unwrap();

    assert_eq!(after.background, None);
    assert_eq!(settings.get_settings().unwrap().background, None);
    assert!(service.list().unwrap().is_empty());
}

#[test]
fn removing_other_image_keeps_selection() {
    let (service, settings) = service();
    let selected = service.add(&PNG).unwrap();
    let other = service.add(&JPEG).unwrap();
    select(&settings, &selected.id);

    let after = service.remove(&other.id).unwrap();

    assert_eq!(after.background, Some(selected.id));
    assert_eq!(service.list().unwrap().len(), 1);
}
