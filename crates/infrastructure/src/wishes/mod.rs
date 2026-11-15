//! Реализации хранилища истории молитв.

// Код модуля лежит в папке модуля вместе с тестами: sqlite_wish_repository/sqlite_wish_repository.rs
#[path = "sqlite_wish_repository/sqlite_wish_repository.rs"]
mod sqlite_wish_repository;

pub use sqlite_wish_repository::SqliteWishRepository;
