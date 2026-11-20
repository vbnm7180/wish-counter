//! Общее состояние приложения, доступное всем командам Tauri.

use std::sync::Arc;

use application::services::{
    BackgroundService, ExportService, GameCatalog, HistoryService, ImageService, ImportService,
    ProfileService, ReferenceUpdater, SettingsService, StatisticsService,
};

/// Сервисы, которые нужны командам интерфейса.
///
/// Tauri хранит этот объект (`Builder::manage`) и передаёт ссылку на него
/// в каждую команду через параметр `tauri::State<AppState>`.
pub struct AppState {
    /// Каталог поддерживаемых игр. В `Arc`, потому что им пользуется
    /// и [`StatisticsService`].
    pub game_catalog: Arc<GameCatalog>,
    /// Сервис пользовательских настроек. В `Arc`: им пользуется и [`BackgroundService`].
    pub settings_service: Arc<SettingsService>,
    /// Картинки фона. В `Arc`: работает в потоках протокола `wcimg` и в диалоге выбора файла.
    pub background_service: Arc<BackgroundService>,
    /// Обновление справочников из сети. В `Arc`, потому что работает и в фоновом потоке.
    pub reference_updater: Arc<ReferenceUpdater>,
    /// Картинки предметов. В `Arc`, потому что работает в потоках протокола `wcimg`.
    pub image_service: Arc<ImageService>,
    /// Экспорт истории молитв в файлы. В `Arc`: работает в отдельном потоке.
    pub export_service: Arc<ExportService>,
    /// История молитв для таблицы. В `Arc`: работает в отдельном потоке.
    pub history_service: Arc<HistoryService>,
    /// Импорт истории молитв из файлов. В `Arc`: работает в отдельном потоке.
    pub import_service: Arc<ImportService>,
    /// Сервис профилей игр (сервер игрока).
    pub profile_service: ProfileService,
    /// Сервис статистики молитв.
    pub statistics_service: StatisticsService,
}
