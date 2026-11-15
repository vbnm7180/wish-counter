//! Сервисы — классы со сценариями использования.
//!
//! Каждый сервис отвечает за одну область (принцип единственной
//! ответственности, буква S в SOLID).

// Код модуля лежит в папке модуля вместе с тестами: game_catalog/game_catalog.rs
#[path = "game_catalog/game_catalog.rs"]
mod game_catalog;
// Код модуля лежит в папке модуля вместе с тестами: settings_service/settings_service.rs
#[path = "settings_service/settings_service.rs"]
mod settings_service;

pub use game_catalog::GameCatalog;
pub use settings_service::SettingsService;
