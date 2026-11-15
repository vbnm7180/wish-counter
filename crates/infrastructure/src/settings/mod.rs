//! Реализации хранилища пользовательских настроек.

// Код модуля лежит в папке модуля вместе с тестами: json_settings_repository/json_settings_repository.rs
#[path = "json_settings_repository/json_settings_repository.rs"]
mod json_settings_repository;

pub use json_settings_repository::JsonSettingsRepository;
