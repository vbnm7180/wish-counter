//! Общее состояние приложения, доступное всем командам Tauri.

use std::sync::Arc;

use application::services::{GameCatalog, ReferenceService, SettingsService, StatisticsService};

/// Сервисы, которые нужны командам интерфейса.
///
/// Tauri хранит этот объект (`Builder::manage`) и передаёт ссылку на него
/// в каждую команду через параметр `tauri::State<AppState>`.
pub struct AppState {
    /// Каталог поддерживаемых игр. В `Arc`, потому что им пользуется
    /// и [`StatisticsService`].
    pub game_catalog: Arc<GameCatalog>,
    /// Сервис пользовательских настроек.
    pub settings_service: SettingsService,
    /// Сервис справочных данных (персонажи, оружие, баннеры).
    pub reference_service: ReferenceService,
    /// Сервис статистики молитв.
    pub statistics_service: StatisticsService,
}
