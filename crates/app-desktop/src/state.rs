//! Общее состояние приложения, доступное всем командам Tauri.

use std::sync::Arc;

use application::services::{
    GameCatalog, ImageService, ProfileService, ReferenceUpdater, SettingsService, StatisticsService,
};

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
    /// Обновление справочников из сети. В `Arc`, потому что работает и в фоновом потоке.
    pub reference_updater: Arc<ReferenceUpdater>,
    /// Картинки предметов. В `Arc`, потому что работает в потоках протокола `wcimg`.
    pub image_service: Arc<ImageService>,
    /// Сервис профилей игр (сервер игрока).
    pub profile_service: ProfileService,
    /// Сервис статистики молитв.
    pub statistics_service: StatisticsService,
}
