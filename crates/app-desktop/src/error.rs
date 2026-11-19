//! Ошибки уровня приложения: запуска и выполнения команд.

use application::ports::RepositoryError;
use application::services::{
    ExportServiceError, ImportServiceError, ProfileError, ReferenceError, StatisticsError,
    UpdateError,
};
use domain::import::ImportError;
use domain::profile::PlayerUidError;
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

impl From<ProfileError> for CommandError {
    /// Преобразует ошибку сервиса профилей в ошибку команды.
    fn from(error: ProfileError) -> Self {
        match error {
            ProfileError::Repository(repository_error) => repository_error.into(),
            ProfileError::UnknownGame(_) => {
                tracing::warn!(%error, "профиль для неизвестной игры");
                Self {
                    code: "unknown_game",
                    message: error.to_string(),
                }
            }
        }
    }
}

impl From<UpdateError> for CommandError {
    /// Преобразует ошибку обновления справочника в ошибку команды.
    fn from(error: UpdateError) -> Self {
        tracing::warn!(%error, "справочник не обновлён");
        let code = match error {
            UpdateError::UnknownGame(_) => "unknown_game",
            UpdateError::Download(_) => "reference_download_failed",
            UpdateError::InvalidData(_) => "reference_invalid",
            UpdateError::Repository(_) => "storage_io",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<tauri::Error> for CommandError {
    /// Внутренняя ошибка Tauri (например, сбой фонового потока).
    fn from(error: tauri::Error) -> Self {
        tracing::error!(%error, "внутренняя ошибка при выполнении команды");
        Self {
            code: "internal",
            message: error.to_string(),
        }
    }
}

impl From<ImportServiceError> for CommandError {
    /// Преобразует ошибку импорта в ошибку команды (код выбирает текст в интерфейсе).
    fn from(error: ImportServiceError) -> Self {
        tracing::warn!(%error, "история не импортирована");
        let code = match &error {
            ImportServiceError::UnknownGame(_) => "unknown_game",
            ImportServiceError::UnsupportedFormat(_) => "import_unsupported_format",
            ImportServiceError::FileTooLarge => "import_file_too_large",
            ImportServiceError::Parse(ImportError::InvalidFile { .. }) => "import_invalid_file",
            ImportServiceError::Parse(ImportError::UnknownItems(_)) => "import_unknown_items",
            ImportServiceError::Repository(_) => "storage_io",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<ExportServiceError> for CommandError {
    /// Преобразует ошибку экспорта в ошибку команды.
    fn from(error: ExportServiceError) -> Self {
        tracing::warn!(%error, "история не экспортирована");
        let code = match &error {
            ExportServiceError::UnknownGame(_) => "unknown_game",
            ExportServiceError::UnsupportedFormat(_) => "export_unsupported_format",
            ExportServiceError::EmptyHistory => "export_empty_history",
            ExportServiceError::Export(_) => "export_failed",
            ExportServiceError::Repository(_) => "storage_io",
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

impl From<PlayerUidError> for CommandError {
    /// Некорректный UID, введённый пользователем.
    fn from(error: PlayerUidError) -> Self {
        Self {
            code: "invalid_uid",
            message: error.to_string(),
        }
    }
}

impl CommandError {
    /// Ошибка записи файла экспорта (путь в лог не пишется — он может содержать имя пользователя).
    pub fn export_write(error: impl std::fmt::Display) -> Self {
        tracing::warn!(%error, "не удалось записать файл экспорта");
        Self {
            code: "export_write_failed",
            message: error.to_string(),
        }
    }
}
