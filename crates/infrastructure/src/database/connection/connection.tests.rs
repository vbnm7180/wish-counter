//! Модульные тесты для `connection.rs`.
//!
//! Подключаются в `connection.rs` строками `#[cfg(test)] #[path = "connection.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;
use crate::database::migrations::{latest_version, schema_version};

#[test]
fn creates_database_file_with_schema() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("test.sqlite");

    let connection = open_database(&path).unwrap();

    assert!(path.is_file());
    assert_eq!(schema_version(&connection).unwrap(), latest_version());
}

#[test]
fn enables_write_ahead_log() {
    let temp = tempfile::tempdir().unwrap();
    let connection = open_database(&temp.path().join("test.sqlite")).unwrap();

    let mode: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();

    assert_eq!(mode.to_lowercase(), "wal");
}

/// Данные сохраняются в файле и видны после повторного открытия.
#[test]
fn data_survives_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("test.sqlite");
    {
        let connection = open_database(&path).unwrap();
        connection
            .execute(
                "INSERT INTO wishes (game_id, wish_id, banner_code, item_id, item_kind, rarity, wish_time)
                 VALUES ('genshin', '1', 301, 'hu_tao', 'character', 5, '2026-01-01 00:00:00')",
                [],
            )
            .unwrap();
    } // здесь соединение закрывается: `connection` выходит из области видимости

    let reopened = open_database(&path).unwrap();
    let count: i64 = reopened
        .query_row("SELECT COUNT(*) FROM wishes", [], |row| row.get(0))
        .unwrap();

    assert_eq!(count, 1);
}

#[test]
fn in_memory_database_has_schema() {
    let connection = open_in_memory_database().unwrap();
    assert_eq!(schema_version(&connection).unwrap(), latest_version());
}
