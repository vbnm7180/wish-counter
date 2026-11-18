//! Интерфейс загрузчика исходных данных справочника.

use domain::reference::{ReferenceSourceSpec, SourceFiles};
use thiserror::Error;

/// Ошибка скачивания (нет сети, сервер недоступен, слишком большой ответ и т.п.).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("не удалось скачать данные: {0}")]
pub struct DownloadError(pub String);

/// Скачивает исходные файлы справочника из репозитория.
pub trait ReferenceDownloader: Send + Sync {
    /// Возвращает идентификатор последней ревизии репозитория.
    /// Это один маленький запрос — по нему понятно, изменились ли данные.
    fn latest_revision(&self, source: &ReferenceSourceSpec) -> Result<String, DownloadError>;

    /// Скачивает все файлы `source.files` указанной ревизии.
    fn download(
        &self,
        source: &ReferenceSourceSpec,
        revision: &str,
    ) -> Result<SourceFiles, DownloadError>;
}
