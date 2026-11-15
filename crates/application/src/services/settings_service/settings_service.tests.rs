//! Модульные тесты для `settings_service.rs`.
//!
//! Подключаются в `settings_service.rs` строками `#[cfg(test)] #[path = "settings_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::sync::Mutex;

use domain::settings::{Language, ThemeId};

use super::*;

/// Хранилище в памяти для тестов. Можно заранее задать результат `load`.
struct InMemoryRepository {
    stored: Mutex<Option<AppSettings>>,
    load_error: Option<fn() -> RepositoryError>,
}

impl InMemoryRepository {
    /// Пустое хранилище без ошибок.
    fn empty() -> Self {
        Self {
            stored: Mutex::new(None),
            load_error: None,
        }
    }

    /// Хранилище, которое при чтении возвращает указанную ошибку.
    fn failing(error: fn() -> RepositoryError) -> Self {
        Self {
            stored: Mutex::new(None),
            load_error: Some(error),
        }
    }
}

impl SettingsRepository for InMemoryRepository {
    fn load(&self) -> Result<Option<AppSettings>, RepositoryError> {
        match self.load_error {
            Some(make_error) => Err(make_error()),
            None => Ok(self.stored.lock().unwrap().clone()),
        }
    }

    fn save(&self, settings: &AppSettings) -> Result<(), RepositoryError> {
        *self.stored.lock().unwrap() = Some(settings.clone());
        Ok(())
    }
}

/// Пример нестандартных настроек для тестов.
fn custom_settings() -> AppSettings {
    AppSettings {
        theme: ThemeId::parse("light").unwrap(),
        language: Language::English,
    }
}

#[test]
fn returns_defaults_when_nothing_saved() {
    let service = SettingsService::new(Arc::new(InMemoryRepository::empty()));
    assert_eq!(service.get_settings().unwrap(), AppSettings::default());
}

#[test]
fn returns_saved_settings_after_update() {
    let service = SettingsService::new(Arc::new(InMemoryRepository::empty()));

    service.update_settings(custom_settings()).unwrap();

    assert_eq!(service.get_settings().unwrap(), custom_settings());
}

#[test]
fn falls_back_to_defaults_when_storage_corrupted() {
    let repository = InMemoryRepository::failing(|| RepositoryError::Corrupted("bad json".into()));
    let service = SettingsService::new(Arc::new(repository));

    assert_eq!(service.get_settings().unwrap(), AppSettings::default());
}

#[test]
fn propagates_io_errors() {
    let repository = InMemoryRepository::failing(|| RepositoryError::Io("disk".into()));
    let service = SettingsService::new(Arc::new(repository));

    assert!(matches!(
        service.get_settings(),
        Err(RepositoryError::Io(_))
    ));
}
