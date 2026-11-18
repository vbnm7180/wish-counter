//! # Слой инфраструктуры Wish Counter
//!
//! Здесь находятся «технические детали»: где лежат файлы, как пишутся логи,
//! как сохраняются данные. Крейт реализует порты (трейты) из крейта
//! `application`.
//!
//! Модули:
//! - [`paths`] — расположение папок приложения (portable-режим и задел под установщик);
//! - [`logging`] — настройка журнала работы (логов);
//! - [`settings`] — хранение пользовательских настроек в JSON-файле;
//! - [`database`] — база SQLite: открытие файла, настройка, миграции схемы;
//! - [`wishes`] — хранение истории молитв в SQLite;
//! - [`profiles`] — профили игр (сервер игрока) в SQLite;
//! - [`reference_cache`] — скачанные обновления справочников (`data/catalog/`);
//! - [`remote`] — скачивание данных: безопасный HTTP-клиент и GitHub.

pub mod database;
// Код модуля лежит в папке модуля вместе с тестами: json_file/json_file.rs
#[path = "json_file/json_file.rs"]
mod json_file;
pub mod logging;
// Код модуля лежит в папке модуля вместе с тестами: paths/paths.rs
#[path = "paths/paths.rs"]
pub mod paths;
pub mod profiles;
pub mod reference_cache;
pub mod remote;
pub mod settings;
pub mod wishes;
