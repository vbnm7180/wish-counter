//! Сервисы — классы со сценариями использования.
//!
//! Каждый сервис отвечает за одну область (принцип единственной
//! ответственности, буква S в SOLID).

// Код модуля лежит в папке модуля вместе с тестами: game_catalog/game_catalog.rs
#[path = "game_catalog/game_catalog.rs"]
mod game_catalog;
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

pub use game_catalog::GameCatalog;
pub use profile_service::{ProfileError, ProfileService};
pub use reference_service::{LatestBanner, ReferenceError, ReferenceService, ReferenceSummary};
pub use reference_updater::{
    ReferenceStatus, ReferenceUpdater, UpdateError, UpdateMode, UpdateOutcome,
};
pub use settings_service::SettingsService;
pub use statistics_service::{GameStatistics, PoolSummary, StatisticsError, StatisticsService};
