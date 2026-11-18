//! # Wish Counter — точка входа десктопного приложения
//!
//! Этот файл только запускает приложение. Вся сборка объектов находится
//! в [`bootstrap`], команды для интерфейса — в [`commands`].

// В релизной сборке не открывать чёрное окно консоли рядом с окном программы.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod background;
mod bootstrap;
mod commands;
mod error;
mod events;
// Код модуля лежит в папке модуля вместе с тестами: image_protocol/image_protocol.rs
#[path = "image_protocol/image_protocol.rs"]
mod image_protocol;
mod state;
mod window;

/// Главная функция: запускает приложение и при фатальной ошибке
/// выводит её и завершает процесс с ненулевым кодом.
fn main() {
    if let Err(error) = bootstrap::run() {
        eprintln!("Wish Counter не смог запуститься: {error}");
        std::process::exit(1);
    }
}
