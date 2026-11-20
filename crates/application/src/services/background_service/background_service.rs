//! Фоны приложения: добавление своих картинок, список, удаление.
//!
//! Картинка проверяется (размер, формат по содержимому) и копируется в хранилище
//! под идентификатором, вычисленным по содержимому. Выбранный фон хранится
//! в настройках (`AppSettings::background`); при удалении выбранной картинки
//! выбор сбрасывается, чтобы настройки не ссылались на несуществующий файл.

use std::sync::Arc;

use domain::background::{BackgroundId, BackgroundImage, ImageFormat};
use domain::settings::AppSettings;
use thiserror::Error;

use super::SettingsService;
use crate::ports::{BackgroundStore, RepositoryError};

/// Максимальный размер картинки фона (20 МБ): ключевые арты 2560×1440 в PNG весят около 5 МБ.
pub const MAX_BACKGROUND_BYTES: u64 = 20 * 1024 * 1024;

/// Ошибка работы с фонами.
#[derive(Debug, Error)]
pub enum BackgroundError {
    /// Файл больше [`MAX_BACKGROUND_BYTES`].
    #[error("картинка больше {} МБ", MAX_BACKGROUND_BYTES / 1024 / 1024)]
    TooLarge,
    /// Файл не PNG, не JPEG и не WebP.
    #[error("формат картинки не поддерживается")]
    UnsupportedFormat,
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Управляет картинками фона.
pub struct BackgroundService {
    store: Arc<dyn BackgroundStore>,
    settings: Arc<SettingsService>,
}

impl BackgroundService {
    /// Создаёт сервис фонов.
    pub fn new(store: Arc<dyn BackgroundStore>, settings: Arc<SettingsService>) -> Self {
        Self { store, settings }
    }

    /// Все добавленные картинки (старые первыми).
    pub fn list(&self) -> Result<Vec<BackgroundImage>, RepositoryError> {
        self.store.list()
    }

    /// Проверяет и сохраняет картинку. Повторное добавление того же файла
    /// возвращает уже сохранённую картинку (идентификатор зависит от содержимого).
    pub fn add(&self, bytes: &[u8]) -> Result<BackgroundImage, BackgroundError> {
        if exceeds_limit(bytes.len()) {
            return Err(BackgroundError::TooLarge);
        }
        let format = ImageFormat::detect(bytes).ok_or(BackgroundError::UnsupportedFormat)?;
        let image = BackgroundImage {
            id: BackgroundId::from_content(bytes),
            format,
        };
        self.store.save(&image, bytes)?;
        tracing::info!(id = %image.id, ?format, "добавлен фон");
        Ok(image)
    }

    /// Картинка и её формат; `None`, если картинки нет.
    pub fn image(
        &self,
        id: &BackgroundId,
    ) -> Result<Option<(ImageFormat, Vec<u8>)>, RepositoryError> {
        Ok(self
            .store
            .load(id)?
            .map(|(image, bytes)| (image.format, bytes)))
    }

    /// Удаляет картинку. Если она была выбрана, фон сбрасывается.
    /// Возвращает настройки после удаления.
    pub fn remove(&self, id: &BackgroundId) -> Result<AppSettings, BackgroundError> {
        self.store.remove(id)?;
        tracing::info!(%id, "фон удалён");
        let settings = self.settings.get_settings()?;
        if settings.background.as_ref() != Some(id) {
            return Ok(settings);
        }
        Ok(self.settings.update_settings(AppSettings {
            background: None,
            ..settings
        })?)
    }
}

/// Больше ли размер допустимого. `usize` → `u64` без потерь: `u64` не меньше `usize`
/// на всех поддерживаемых платформах, а `try_from` защищает и от остальных.
fn exceeds_limit(length: usize) -> bool {
    u64::try_from(length).map_or(true, |length| length > MAX_BACKGROUND_BYTES)
}

// Тесты лежат рядом, в файле `background_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "background_service.tests.rs"]
mod tests;
