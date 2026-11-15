//! Ошибки уровня приложения: запуска и выполнения команд.

use application::ports::RepositoryError;
use application::services::{ReferenceError, StatisticsError};
use infrastructure::database::DatabaseError;
use infrastructure::logging::LoggingError;
use infrastructure::paths::PathsError;
use serde::Serialize;
use thiserror::Error;

/// Ошибка, из-за которой приложение не может запуститься.
#[derive(Debug, Error)]
pub enum StartupError {
    /// Не удалось определить пути приложения.
    #[error(transparent)]
    Paths(#[from] PathsError),
    /// Не удалось создать папку данных (например, нет прав на запись).
    #[error("не удалось создать папку данных: {0}")]
    DataDirectory(#[source] std::io::Error),
    /// Не удалось открыть базу данных или обновить её схему.
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Не удалось включить логирование.
    #[error(transparent)]
    Logging(#[from] LoggingError),
    /// Ошибка самого Tauri (окно, WebView2 и т.п.).
    #[error("ошибка Tauri: {0}")]
    Tauri(#[from] tauri::Error),
}

/// Ошибка команды, которую получает интерфейс.
///
/// Отправляется в JavaScript как объект `{ code, message }`.
/// `code` — стабильный машинный код (по нему интерфейс выбирает
/// переведённый текст), `message` — подробности для логов и отладки.
#[derive(Debug, Serialize)]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl From<RepositoryError> for CommandError {
    /// Преобразует ошибку хранилища в ошибку команды и пишет её в лог.
    fn from(error: RepositoryError) -> Self {
        tracing::error!(%error, "ошибка хранилища при выполнении команды");
        let code = match error {
            RepositoryError::Io(_) => "storage_io",
            RepositoryError::Corrupted(_) => "storage_corrupted",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<StatisticsError> for CommandError {
    /// Преобразует ошибку расчёта статистики в ошибку команды.
    fn from(error: StatisticsError) -> Self {
        match error {
            StatisticsError::Repository(repository_error) => repository_error.into(),
            StatisticsError::UnknownGame(_) => {
                tracing::warn!(%error, "запрошена неизвестная игра");
                Self {
                    code: "unknown_game",
                    message: error.to_string(),
                }
            }
        }
    }
}

impl From<ReferenceError> for CommandError {
    /// Преобразует ошибку справочника в ошибку команды.
    fn from(error: ReferenceError) -> Self {
        tracing::warn!(%error, "справочник недоступен");
        Self {
            code: "unknown_game",
            message: error.to_string(),
        }
    }
}
