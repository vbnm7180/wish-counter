//! Работа с файлами хранилищ: чтение, безопасная (атомарная) запись,
//! проверка имён файлов.
//!
//! Используется хранилищами, которые держат данные в файлах (настройки,
//! кэш справочников, кэш картинок): одна реализация вместо копий в каждом.

use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use application::ports::RepositoryError;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Окончание имени временного файла, в который сначала пишутся данные.
const TEMPORARY_SUFFIX: &str = ".tmp";

/// Читает файл целиком. Нет файла — `Ok(None)`.
pub fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, RepositoryError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(error)),
    }
}

/// Читает JSON-файл. Нет файла — `Ok(None)`, неверный JSON — `Corrupted`.
///
/// `T: DeserializeOwned` — «любой тип, который serde умеет прочитать
/// и который владеет своими данными» (не ссылается на исходный текст).
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, RepositoryError> {
    let Some(bytes) = read_bytes(path)? else {
        return Ok(None);
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| RepositoryError::Corrupted(error.to_string()))
}

/// Записывает байты в файл «атомарно»: сначала во временный файл рядом,
/// затем переименовывает его поверх основного. Если программа упадёт
/// посреди записи, старый файл останется целым, а «половинчатого» не будет.
pub fn write_bytes_atomically(path: &Path, bytes: &[u8]) -> Result<(), RepositoryError> {
    let mut temporary_name = path.as_os_str().to_owned();
    temporary_name.push(TEMPORARY_SUFFIX);
    let temporary_path = Path::new(&temporary_name);
    fs::write(temporary_path, bytes).map_err(io_error)?;
    fs::rename(temporary_path, path).map_err(io_error)
}

/// Записывает значение в JSON-файл атомарно (см. [`write_bytes_atomically`]).
pub fn write_json_atomically<T: Serialize>(path: &Path, value: &T) -> Result<(), RepositoryError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| RepositoryError::Corrupted(error.to_string()))?;
    write_bytes_atomically(path, json.as_bytes())
}

/// Можно ли использовать строку как часть имени файла без риска выйти из папки:
/// только `a-z`, `0-9`, `_`, `-`, не пустая.
///
/// Идентификаторы игр и предметов попадают в пути кэшей, а приходят они в том
/// числе из интерфейса — поэтому `../` и подобное должно отклоняться.
pub fn is_safe_file_stem(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'))
}

/// Ошибка «недопустимое имя» для [`is_safe_file_stem`].
pub fn unsafe_name_error(kind: &str, value: &str) -> RepositoryError {
    RepositoryError::Io(format!("недопустимый идентификатор {kind} «{value}»"))
}

/// Превращает ошибку ввода-вывода в ошибку хранилища.
fn io_error(error: io::Error) -> RepositoryError {
    RepositoryError::Io(error.to_string())
}

// Тесты лежат рядом, в файле `files.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "files.tests.rs"]
mod tests;
