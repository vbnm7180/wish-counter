//! Реализации хранилища профилей игр.

// Код модуля лежит в папке модуля вместе с тестами: sqlite_profile_repository/sqlite_profile_repository.rs
#[path = "sqlite_profile_repository/sqlite_profile_repository.rs"]
mod sqlite_profile_repository;

pub use sqlite_profile_repository::SqliteProfileRepository;
