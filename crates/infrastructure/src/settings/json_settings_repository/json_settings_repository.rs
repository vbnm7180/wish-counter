//! Хранилище настроек в JSON-файле.

use std::path::PathBuf;

use application::ports::{RepositoryError, SettingsRepository};
use domain::settings::AppSettings;

use crate::json_file::{read_json, write_json_atomically};

/// Сохраняет настройки в человекочитаемый JSON-файл (`data/settings.json`).
///
/// Настройки хранятся отдельно от базы молитв: файл можно открыть
/// в блокноте, а повреждение базы не сбросит тему и язык.
pub struct JsonSettingsRepository {
    file_path: PathBuf,
}

impl JsonSettingsRepository {
    /// Создаёт хранилище, работающее с указанным файлом.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }
}

impl SettingsRepository for JsonSettingsRepository {
    /// Читает файл настроек. Если файла нет — возвращает `None`.
    fn load(&self) -> Result<Option<AppSettings>, RepositoryError> {
        read_json(&self.file_path)
    }

    /// Записывает настройки «атомарно»: сначала во временный файл,
    /// затем переименовывает его поверх основного. Если программа упадёт
    /// посреди записи, старый файл останется целым.
    fn save(&self, settings: &AppSettings) -> Result<(), RepositoryError> {
        write_json_atomically(&self.file_path, settings)
    }
}

// Тесты лежат рядом, в файле `json_settings_repository.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "json_settings_repository.tests.rs"]
mod tests;
