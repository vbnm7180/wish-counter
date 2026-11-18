//! Чтение и безопасная запись JSON-файлов.
//!
//! Используется хранилищами, которые держат данные в JSON (настройки,
//! кэш справочников): одна реализация вместо копий в каждом хранилище.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use application::ports::RepositoryError;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Окончание имени временного файла, в который сначала пишутся данные.
const TEMPORARY_SUFFIX: &str = ".tmp";

/// Читает JSON-файл. Нет файла — `Ok(None)`, неверный JSON — `Corrupted`.
///
/// `T: DeserializeOwned` — «любой тип, который serde умеет прочитать
/// и который владеет своими данными» (не ссылается на исходный текст).
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, RepositoryError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(error)),
    };
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| RepositoryError::Corrupted(error.to_string()))
}

/// Записывает значение в JSON-файл «атомарно»: сначала во временный файл
/// рядом, затем переименовывает его поверх основного. Если программа упадёт
/// посреди записи, старый файл останется целым.
pub fn write_json_atomically<T: Serialize>(path: &Path, value: &T) -> Result<(), RepositoryError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| RepositoryError::Corrupted(error.to_string()))?;
    let mut temporary_name = path.as_os_str().to_owned();
    temporary_name.push(TEMPORARY_SUFFIX);
    let temporary_path = Path::new(&temporary_name);
    fs::write(temporary_path, json).map_err(io_error)?;
    fs::rename(temporary_path, path).map_err(io_error)
}

/// Превращает ошибку ввода-вывода в ошибку хранилища.
fn io_error(error: io::Error) -> RepositoryError {
    RepositoryError::Io(error.to_string())
}

// Тесты лежат рядом, в файле `json_file.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "json_file.tests.rs"]
mod tests;
