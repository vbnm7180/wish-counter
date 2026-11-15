//! Встроенный в программу снимок справочника Genshin Impact.
//!
//! Файл `crates/games/genshin/data/reference.json` создаётся командой
//! `cargo run -p data-sync` (см. docs/development.md) и встраивается в `.exe`
//! при компиляции. Благодаря этому приложение знает персонажей и баннеры
//! без интернета с первого запуска.

use domain::reference::ReferenceSnapshot;

/// Текст встроенного справочника.
///
/// `include_str!` читает файл при компиляции; путь — относительно этого файла.
const EMBEDDED_REFERENCE_JSON: &str = include_str!("../../data/reference.json");

/// Разбирает встроенный справочник.
///
/// Корректность файла проверяет тест `embedded_reference_is_valid`,
/// поэтому ошибка здесь возможна только при сборке с непроверенным файлом.
pub fn embedded_reference() -> ReferenceSnapshot {
    serde_json::from_str(EMBEDDED_REFERENCE_JSON)
        .expect("встроенный справочник data/reference.json повреждён (см. тест embedded_reference_is_valid)")
}

// Тесты лежат рядом, в файле `reference.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "reference.tests.rs"]
mod tests;
