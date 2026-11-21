//! Порты — интерфейсы (трейты) для внешнего мира.
//!
//! Слой приложения объявляет, *что* ему нужно (например, «загрузить
//! настройки»), а инфраструктура решает, *как* это сделать.

mod background_store;
mod game_api_client;
mod image_store;
mod profile_repository;
mod reference_cache;
mod reference_downloader;
mod settings_repository;
mod wish_repository;

pub use background_store::BackgroundStore;
pub use game_api_client::{GameApiClient, GameApiError as GameApiClientError};
pub use image_store::{ImageCache, ImageDownloader};
pub use profile_repository::ProfileRepository;
pub use reference_cache::ReferenceCache;
pub use reference_downloader::{DownloadError, ReferenceDownloader};
pub use settings_repository::{RepositoryError, SettingsRepository};
pub use wish_repository::WishRepository;
