//! Реализации хранилища истории молитв.

// Код модуля лежит в папке модуля вместе с тестами: in_memory_wish_repository/in_memory_wish_repository.rs
#[path = "in_memory_wish_repository/in_memory_wish_repository.rs"]
mod in_memory_wish_repository;

pub use in_memory_wish_repository::InMemoryWishRepository;
