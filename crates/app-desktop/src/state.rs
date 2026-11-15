//! Общее состояние приложения, доступное всем командам Tauri.

use application::services::{GameCatalog, SettingsService};

/// Сервисы, которые нужны командам интерфейса.
///
/// Tauri хранит этот объект (`Builder::manage`) и передаёт ссылку на него
/// в каждую команду через параметр `tauri::State<AppState>`.
pub struct AppState {
    /// Каталог поддерживаемых игр.
    pub game_catalog: GameCatalog,
    /// Сервис пользовательских настроек.
    pub settings_service: SettingsService,
}
