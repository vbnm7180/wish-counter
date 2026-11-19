//! Данные Paimon.moe: где они лежат и как их перевести в справочник.
//!
//! Источник: <https://github.com/MadeBaruna/paimon-moe> (лицензия MIT).
//! Автор Paimon.moe обновляет эти файлы вручную в день выхода новых баннеров.

// Код модуля лежит в папке модуля вместе с тестами: backup/backup.rs
#[path = "backup/backup.rs"]
pub mod backup;
// Код модуля лежит в папке модуля вместе с тестами: converter/converter.rs
#[path = "converter/converter.rs"]
pub mod converter;
// Код модуля лежит в папке модуля вместе с тестами: js_literal/js_literal.rs
#[path = "js_literal/js_literal.rs"]
pub mod js_literal;
// Код модуля лежит в папке модуля вместе с тестами: remote/remote.rs
#[path = "remote/remote.rs"]
pub mod remote;

/// Владелец репозитория Paimon.moe на GitHub.
pub const REPOSITORY_OWNER: &str = "MadeBaruna";
/// Имя репозитория Paimon.moe на GitHub.
pub const REPOSITORY_NAME: &str = "paimon-moe";

/// Пути к нужным файлам внутри репозитория Paimon.moe.
pub mod source_files {
    /// Расписание баннеров.
    pub const BANNERS: &str = "src/data/banners.js";
    /// Персонажи.
    pub const CHARACTERS: &str = "src/data/characters.js";
    /// Оружие.
    pub const WEAPONS: &str = "src/data/weaponList.js";
    /// Русские названия предметов.
    pub const NAMES_RU: &str = "src/locales/items/ru.json";
    /// Папка картинок персонажей (`<папка>/<id>.png`).
    pub const CHARACTER_IMAGES: &str = "static/images/characters";
    /// Папка картинок оружия (`<папка>/<id>.png`).
    pub const WEAPON_IMAGES: &str = "static/images/weapons";
}
