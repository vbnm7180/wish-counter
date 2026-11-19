//! Импорт истории молитв из файлов других сервисов (Paimon.moe, позже UIGF).
//!
//! Каждый формат — реализация трейта [`HistoryParser`] в модуле игры:
//! формат файла знает только он. Общий результат разбора — [`ParsedHistory`].

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::reference::ReferenceData;
use crate::region::ServerRegion;
use crate::wish::Wish;

/// Идентификатор формата файла, например `"paimon-moe"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImportFormatId(String);

impl ImportFormatId {
    /// Создаёт идентификатор формата.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Возвращает идентификатор как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Результат разбора файла истории.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHistory {
    /// Молитвы в хронологическом порядке (как требует `WishRepository`).
    pub wishes: Vec<Wish>,
    /// Сервер игрока, если он записан в файле.
    pub server_region: Option<ServerRegion>,
    /// Сколько молитв в файле было повторами уже прочитанных (пропущены).
    pub duplicates_skipped: usize,
}

/// Ошибка разбора файла истории.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ImportError {
    /// Файл — не JSON или не того формата.
    #[error("файл не похож на {format}: {reason}")]
    InvalidFile {
        /// Ожидаемый формат.
        format: String,
        /// Подробности.
        reason: String,
    },
    /// В файле есть предметы, которых нет в справочнике (нельзя определить редкость).
    /// Обычно это значит, что справочник устарел.
    #[error("в файле есть неизвестные предметы: {0:?}")]
    UnknownItems(Vec<String>),
}

/// Разборщик одного формата файла истории для одной игры.
pub trait HistoryParser: Send + Sync {
    /// Идентификатор формата.
    fn format_id(&self) -> ImportFormatId;

    /// Разбирает содержимое файла. Справочник нужен, чтобы узнать
    /// редкость предметов (в некоторых форматах её нет).
    fn parse(&self, content: &str, reference: &ReferenceData)
    -> Result<ParsedHistory, ImportError>;
}
