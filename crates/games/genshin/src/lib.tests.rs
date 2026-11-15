//! Модульные тесты для `lib.rs`.
//!
//! Подключаются в `lib.rs` строками `#[cfg(test)] #[path = "lib.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn module_reports_genshin_id() {
    assert_eq!(GenshinModule.info().id.as_str(), GENSHIN_GAME_ID);
}
