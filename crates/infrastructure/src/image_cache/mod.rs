//! Хранилище картинок предметов.

// Код модуля лежит в папке модуля вместе с тестами: file_image_cache/file_image_cache.rs
#[path = "file_image_cache/file_image_cache.rs"]
mod file_image_cache;

pub use file_image_cache::FileImageCache;
