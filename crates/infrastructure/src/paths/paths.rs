//! Расположение папок и файлов приложения.
//!
//! Приложение portable: по умолчанию все данные (настройки, база, кэш картинок,
//! логи) лежат в папке `data/` рядом с `.exe`. Такую папку можно скопировать
//! на флешку вместе с программой.
//!
//! Для будущей версии с установщиком предусмотрен режим
//! [`StorageMode::UserProfile`]: программа в `Program Files` не может писать
//! рядом с собой, поэтому данные хранятся в профиле пользователя (`%APPDATA%`).

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Имя папки с данными в portable-режиме.
const PORTABLE_DATA_DIR_NAME: &str = "data";
/// Имя папки приложения внутри профиля пользователя.
const USER_PROFILE_DIR_NAME: &str = "WishCounter";
/// Имя папки с логами внутри папки данных.
const LOGS_DIR_NAME: &str = "logs";
/// Имя файла настроек внутри папки данных.
const SETTINGS_FILE_NAME: &str = "settings.json";
/// Путь к кэшу картинок внутри папки данных.
const IMAGES_DIR: [&str; 2] = ["cache", "images"];
/// Имя папки картинок фона (добавленных пользователем) внутри папки данных.
const BACKGROUNDS_DIR_NAME: &str = "backgrounds";
/// Имя папки скачанных справочников внутри папки данных.
const CATALOG_DIR_NAME: &str = "catalog";
/// Имя папки служебных данных WebView2 (кэш движка окна) внутри папки данных.
const WEBVIEW_DIR_NAME: &str = "webview";
/// Имя файла базы данных SQLite внутри папки данных.
const DATABASE_FILE_NAME: &str = "wish-counter.sqlite";

/// Где хранить данные приложения.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMode {
    /// Рядом с исполняемым файлом (portable-версия).
    Portable,
    /// В профиле пользователя (версия с установщиком, на будущее).
    UserProfile,
}

/// Ошибка определения путей приложения.
#[derive(Debug, Error)]
pub enum PathsError {
    /// Не удалось узнать путь к запущенному `.exe`.
    #[error("не удалось определить путь к исполняемому файлу: {0}")]
    ExecutableLocation(#[source] io::Error),
    /// У пути к `.exe` нет родительской папки (теоретически невозможно).
    #[error("у исполняемого файла нет родительской папки")]
    NoExecutableDirectory,
    /// Не задана переменная окружения с профилем пользователя.
    #[error("не найдена папка профиля пользователя")]
    NoUserProfile,
}

/// Набор путей, которыми пользуется приложение.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Корневая папка данных.
    pub data_dir: PathBuf,
    /// Папка с файлами логов.
    pub logs_dir: PathBuf,
    /// Файл с пользовательскими настройками.
    pub settings_file: PathBuf,
    /// Файл базы данных SQLite с историей молитв.
    pub database_file: PathBuf,
    /// Папка скачанных обновлений справочников.
    pub catalog_dir: PathBuf,
    /// Папка кэша картинок предметов (`data/cache/images`).
    pub images_dir: PathBuf,
    /// Картинки фона, добавленные пользователем.
    pub backgrounds_dir: PathBuf,
    /// Папка служебных данных WebView2. Без явного указания WebView2 пишет
    /// в `%LOCALAPPDATA%`, а portable-приложение не должно оставлять следов вне своей папки.
    pub webview_dir: PathBuf,
}

impl AppPaths {
    /// Определяет пути для указанного режима хранения.
    pub fn resolve(mode: StorageMode) -> Result<Self, PathsError> {
        let data_dir = match mode {
            StorageMode::Portable => portable_data_dir()?,
            StorageMode::UserProfile => user_profile_data_dir()?,
        };
        Ok(Self::from_data_dir(data_dir))
    }

    /// Строит все пути от заданной корневой папки данных.
    ///
    /// Полезно в тестах: можно передать временную папку.
    pub fn from_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        Self {
            logs_dir: data_dir.join(LOGS_DIR_NAME),
            settings_file: data_dir.join(SETTINGS_FILE_NAME),
            database_file: data_dir.join(DATABASE_FILE_NAME),
            catalog_dir: data_dir.join(CATALOG_DIR_NAME),
            backgrounds_dir: data_dir.join(BACKGROUNDS_DIR_NAME),
            images_dir: IMAGES_DIR
                .iter()
                .fold(data_dir.clone(), |path, part| path.join(part)),
            webview_dir: data_dir.join(WEBVIEW_DIR_NAME),
            data_dir,
        }
    }

    /// Создаёт папки приложения, если их ещё нет.
    pub fn ensure_directories(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        std::fs::create_dir_all(&self.catalog_dir)?;
        std::fs::create_dir_all(&self.images_dir)?;
        std::fs::create_dir_all(&self.backgrounds_dir)?;
        Ok(())
    }
}

/// Возвращает `<папка с .exe>/data`.
fn portable_data_dir() -> Result<PathBuf, PathsError> {
    let executable = std::env::current_exe().map_err(PathsError::ExecutableLocation)?;
    let executable_dir = executable
        .parent()
        .ok_or(PathsError::NoExecutableDirectory)?;
    Ok(executable_dir.join(PORTABLE_DATA_DIR_NAME))
}

/// Возвращает папку данных в профиле пользователя.
///
/// Windows: `%APPDATA%\WishCounter`; другие ОС: `$HOME/.local/share/WishCounter`.
fn user_profile_data_dir() -> Result<PathBuf, PathsError> {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        return Ok(Path::new(&app_data).join(USER_PROFILE_DIR_NAME));
    }
    let home = std::env::var_os("HOME").ok_or(PathsError::NoUserProfile)?;
    Ok(Path::new(&home)
        .join(".local")
        .join("share")
        .join(USER_PROFILE_DIR_NAME))
}

// Тесты лежат рядом, в файле `paths.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "paths.tests.rs"]
mod tests;
