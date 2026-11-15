//! Открытие и настройка соединения с базой SQLite.

use std::path::Path;

use rusqlite::Connection;

use super::DatabaseError;
use super::migrations::apply_migrations;

/// Сколько миллисекунд ждать, если файл базы временно занят.
const BUSY_TIMEOUT_MS: u32 = 5_000;

/// Настройки, которые применяются к каждому соединению.
///
/// - `journal_mode = WAL` — журнал предзаписи: запись не блокирует чтение,
///   а сбой во время записи не портит базу;
/// - `synchronous = NORMAL` — в режиме WAL это надёжно и заметно быстрее, чем `FULL`;
/// - `foreign_keys = ON` — проверка связей между таблицами (по умолчанию в SQLite выключена);
/// - `busy_timeout` — подождать, а не сразу вернуть ошибку, если база занята.
fn configure(connection: &Connection) -> Result<(), DatabaseError> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "busy_timeout", BUSY_TIMEOUT_MS)?;
    Ok(())
}

/// Открывает (или создаёт) файл базы, настраивает соединение
/// и применяет недостающие миграции.
pub fn open_database(path: &Path) -> Result<Connection, DatabaseError> {
    let mut connection = Connection::open(path)?;
    configure(&connection)?;
    apply_migrations(&mut connection)?;
    tracing::info!(path = %path.display(), "база данных открыта");
    Ok(connection)
}

/// Открывает временную базу в оперативной памяти с актуальной схемой.
/// Используется в тестах: база исчезает вместе с соединением.
pub fn open_in_memory_database() -> Result<Connection, DatabaseError> {
    let mut connection = Connection::open_in_memory()?;
    configure(&connection)?;
    apply_migrations(&mut connection)?;
    Ok(connection)
}

// Тесты лежат рядом, в файле `connection.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "connection.tests.rs"]
mod tests;
