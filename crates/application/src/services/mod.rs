//! Сервисы — классы со сценариями использования.
//!
//! Каждый сервис отвечает за одну область (принцип единственной
//! ответственности, буква S в SOLID).

// Код модуля лежит в папке модуля вместе с тестами: background_service/background_service.rs
#[path = "background_service/background_service.rs"]
mod background_service;
// Код модуля лежит в папке модуля вместе с тестами: clear_game_service/clear_game_service.rs
#[path = "clear_game_service/clear_game_service.rs"]
mod clear_game_service;
// Код модуля лежит в папке модуля вместе с тестами: export_service/export_service.rs
#[path = "export_service/export_service.rs"]
mod export_service;
// Код модуля лежит в папке модуля вместе с тестами: game_api_import_service/game_api_import_service.rs
#[path = "game_api_import_service/game_api_import_service.rs"]
mod game_api_import_service;
// Код модуля лежит в папке модуля вместе с тестами: game_catalog/game_catalog.rs
#[path = "game_catalog/game_catalog.rs"]
mod game_catalog;
// Код модуля лежит в папке модуля вместе с тестами: history_service/history_service.rs
#[path = "history_service/history_service.rs"]
mod history_service;
// Код модуля лежит в папке модуля вместе с тестами: image_service/image_service.rs
#[path = "image_service/image_service.rs"]
mod image_service;
// Код модуля лежит в папке модуля вместе с тестами: import_service/import_service.rs
#[path = "import_service/import_service.rs"]
mod import_service;
// Код модуля лежит в папке модуля вместе с тестами: profile_service/profile_service.rs
#[path = "profile_service/profile_service.rs"]
mod profile_service;
// Код модуля лежит в папке модуля вместе с тестами: reference_service/reference_service.rs
#[path = "reference_service/reference_service.rs"]
mod reference_service;
// Код модуля лежит в папке модуля вместе с тестами: reference_updater/reference_updater.rs
#[path = "reference_updater/reference_updater.rs"]
mod reference_updater;
// Код модуля лежит в папке модуля вместе с тестами: settings_service/settings_service.rs
#[path = "settings_service/settings_service.rs"]
mod settings_service;
// Код модуля лежит в папке модуля вместе с тестами: statistics_service/statistics_service.rs
#[path = "statistics_service/statistics_service.rs"]
mod statistics_service;

pub use background_service::{BackgroundError, BackgroundService, MAX_BACKGROUND_BYTES};
pub use clear_game_service::{ClearGameError, ClearGameService, ClearSummary};
pub use export_service::{ExportService, ExportServiceError, ExportedFile};
pub use game_api_import_service::{
    AuthKeyParams, GameApiImportError, GameApiImportService, GameApiImportSummary,
};
pub use game_catalog::GameCatalog;
pub use history_service::{HistoryError, HistoryService, PoolHistory, WishRow};
pub use image_service::{ImageError, ImageService};
pub use import_service::{ImportService, ImportServiceError, ImportSummary, MAX_IMPORT_BYTES};
pub use profile_service::{ProfileError, ProfileService};
pub use reference_service::{
    FeaturedItem, LatestBanner, ReferenceError, ReferenceService, ReferenceSummary,
};
pub use reference_updater::{
    ReferenceStatus, ReferenceUpdater, UpdateError, UpdateMode, UpdateOutcome,
};
pub use settings_service::SettingsService;
pub use statistics_service::{GameStatistics, PoolSummary, StatisticsError, StatisticsService};
