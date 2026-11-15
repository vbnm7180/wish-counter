//! Порты — интерфейсы (трейты) для внешнего мира.
//!
//! Слой приложения объявляет, *что* ему нужно (например, «загрузить
//! настройки»), а инфраструктура решает, *как* это сделать.

mod settings_repository;

pub use settings_repository::{RepositoryError, SettingsRepository};
