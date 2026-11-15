//! Хранилище настроек в JSON-файле.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use application::ports::{RepositoryError, SettingsRepository};
use domain::settings::AppSettings;

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

    /// Путь к временному файлу, используемому при безопасной записи.
    fn temporary_file_path(&self) -> PathBuf {
        self.file_path.with_extension("json.tmp")
    }
}

impl SettingsRepository for JsonSettingsRepository {
    /// Читает файл настроек. Если файла нет — возвращает `None`.
    fn load(&self) -> Result<Option<AppSettings>, RepositoryError> {
        let content = match fs::read_to_string(&self.file_path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(error)),
        };
        serde_json::from_str(&content)
            .map(Some)
            .map_err(|error| RepositoryError::Corrupted(error.to_string()))
    }

    /// Записывает настройки «атомарно»: сначала во временный файл,
    /// затем переименовывает его поверх основного. Если программа упадёт
    /// посреди записи, старый файл останется целым.
    fn save(&self, settings: &AppSettings) -> Result<(), RepositoryError> {
        let json = serde_json::to_string_pretty(settings)
            .map_err(|error| RepositoryError::Corrupted(error.to_string()))?;
        let temporary_path = self.temporary_file_path();
        fs::write(&temporary_path, json).map_err(io_error)?;
        fs::rename(&temporary_path, &self.file_path).map_err(io_error)
    }
}

/// Превращает ошибку ввода-вывода в ошибку хранилища.
fn io_error(error: io::Error) -> RepositoryError {
    RepositoryError::Io(error.to_string())
}

// Тесты лежат рядом, в файле `json_settings_repository.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "json_settings_repository.tests.rs"]
mod tests;
