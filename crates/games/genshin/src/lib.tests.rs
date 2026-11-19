//! Модульные тесты для `lib.rs`.
//!
//! Подключаются в `lib.rs` строками `#[cfg(test)] #[path = "lib.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Первые 4 байта любого PNG-файла.
const PNG_PREFIX: [u8; 4] = [0x89, b'P', b'N', b'G'];

#[test]
fn module_reports_genshin_id() {
    assert_eq!(GenshinModule.info().id.as_str(), GENSHIN_GAME_ID);
}

#[test]
fn module_exposes_five_pools() {
    assert_eq!(GenshinModule.pools().len(), 5);
}

/// Встроенная иконка — настоящий PNG.
#[test]
fn icon_is_png() {
    let icon = GenshinModule.icon_png().expect("иконка встроена");
    assert!(icon.starts_with(&PNG_PREFIX));
}
