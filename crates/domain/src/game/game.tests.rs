//! Модульные тесты для `game.rs`.
//!
//! Подключаются в `game.rs` строками `#[cfg(test)] #[path = "game.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Идентификатор сериализуется в JSON как обычная строка.
#[test]
fn game_id_serializes_as_plain_string() {
    let id = GameId::new("genshin");

    let json = serde_json::to_string(&id).expect("сериализация не должна падать");

    assert_eq!(json, "\"genshin\"");
}

/// `Display` выводит идентификатор без кавычек и обёрток.
#[test]
fn game_id_displays_raw_value() {
    assert_eq!(GameId::new("genshin").to_string(), "genshin");
}
