//! Команды Tauri — функции Rust, которые интерфейс вызывает через `invoke`.
//!
//! Команды «тонкие»: они только достают нужный сервис из состояния
//! и вызывают его. Вся логика находится в слое `application`.

pub mod games;
pub mod history;
pub mod reference;
pub mod settings;
