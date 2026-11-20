//! Модульные тесты для `json_settings_repository.rs`.
//!
//! Подключаются в `json_settings_repository.rs` строками `#[cfg(test)] #[path = "json_settings_repository.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::fs;

use domain::settings::{Language, ThemeId};

use super::*;

/// Создаёт хранилище во временной папке. Папка удаляется,
/// когда возвращённый `TempDir` выходит из области видимости.
fn repository_in_temp_dir() -> (tempfile::TempDir, JsonSettingsRepository) {
    let temp = tempfile::tempdir().unwrap();
    let repository = JsonSettingsRepository::new(temp.path().join("settings.json"));
    (temp, repository)
}

#[test]
fn load_returns_none_when_file_missing() {
    let (_temp, repository) = repository_in_temp_dir();
    assert!(repository.load().unwrap().is_none());
}

#[test]
fn saved_settings_can_be_loaded() {
    let (_temp, repository) = repository_in_temp_dir();
    let settings = AppSettings {
        theme: ThemeId::parse("light").unwrap(),
        language: Language::English,
        background: None,
    };

    repository.save(&settings).unwrap();

    assert_eq!(repository.load().unwrap(), Some(settings));
}

#[test]
fn invalid_json_is_reported_as_corrupted() {
    let (_temp, repository) = repository_in_temp_dir();
    fs::write(&repository.file_path, "{ not json").unwrap();

    assert!(matches!(
        repository.load(),
        Err(RepositoryError::Corrupted(_))
    ));
}
