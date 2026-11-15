//! Модульные тесты для `migrations.rs`.
//!
//! Подключаются в `migrations.rs` строками `#[cfg(test)] #[path = "migrations.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Пустая база в памяти (без миграций).
fn empty_database() -> Connection {
    Connection::open_in_memory().unwrap()
}

/// Проверяет, есть ли в базе таблица с таким именем.
fn table_exists(connection: &Connection, table: &str) -> bool {
    connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
        == 1
}

#[test]
fn fresh_database_gets_latest_version() {
    let mut connection = empty_database();

    apply_migrations(&mut connection).unwrap();

    assert_eq!(schema_version(&connection).unwrap(), latest_version());
    assert!(table_exists(&connection, "wishes"));
}

/// Повторный запуск ничего не применяет повторно и не падает.
#[test]
fn applying_twice_is_harmless() {
    let mut connection = empty_database();

    apply_migrations(&mut connection).unwrap();
    apply_migrations(&mut connection).unwrap();

    assert_eq!(schema_version(&connection).unwrap(), latest_version());
}

/// База от более новой версии приложения не открывается: старый код мог бы её испортить.
#[test]
fn newer_schema_is_rejected() {
    let mut connection = empty_database();
    connection
        .pragma_update(None, "user_version", latest_version() + 1)
        .unwrap();

    let result = apply_migrations(&mut connection);

    assert!(matches!(result, Err(DatabaseError::NewerSchema { .. })));
}
