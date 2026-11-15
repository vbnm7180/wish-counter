//! База данных SQLite: открытие файла, настройка и миграции схемы.
//!
//! Файл базы — `data/wish-counter.sqlite` (см. `paths`). Работа с таблицами
//! находится в хранилищах (например, `wishes::SqliteWishRepository`).

use thiserror::Error;

// Код модуля лежит в папке модуля вместе с тестами: connection/connection.rs
#[path = "connection/connection.rs"]
mod connection;
// Код модуля лежит в папке модуля вместе с тестами: migrations/migrations.rs
#[path = "migrations/migrations.rs"]
mod migrations;

pub use connection::{open_database, open_in_memory_database};

/// Ошибка открытия базы данных или применения миграций.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Ошибка самой SQLite (файл недоступен, повреждён, ошибка в SQL).
    ///
    /// `#[from]` позволяет писать `?` после вызовов rusqlite:
    /// их ошибка автоматически превращается в этот вариант.
    #[error("ошибка базы данных: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// База создана более новой версией приложения.
    #[error(
        "база данных создана более новой версией Wish Counter (схема {found}, поддерживается {supported}); обновите приложение"
    )]
    NewerSchema {
        /// Версия схемы в файле.
        found: u32,
        /// Последняя версия, известная этой сборке.
        supported: u32,
    },
}
