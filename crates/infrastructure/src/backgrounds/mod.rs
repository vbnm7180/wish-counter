//! Хранилище картинок фона, добавленных пользователем.

// Код модуля лежит в папке модуля вместе с тестами: file_background_store/file_background_store.rs
#[path = "file_background_store/file_background_store.rs"]
mod file_background_store;

pub use file_background_store::FileBackgroundStore;
