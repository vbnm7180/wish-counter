//! Импорт и экспорт истории молитв в форматах других сервисов (Paimon.moe, позже UIGF).
//!
//! Каждый формат — реализации трейтов [`HistoryParser`] (чтение) и
//! [`HistoryExporter`] (запись) в модуле игры: формат файла знает только он.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pity::rules::PoolDefinition;
use crate::profile::{GameProfile, PlayerUid};
use crate::reference::ReferenceData;
use crate::region::ServerRegion;
use crate::wish::Wish;

/// Идентификатор формата файла истории (для импорта и экспорта), например `"paimon-moe"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HistoryFormatId(String);

impl HistoryFormatId {
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
    /// UID игрока, если он записан в файле (и корректен).
    pub uid: Option<PlayerUid>,
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
    fn format_id(&self) -> HistoryFormatId;

    /// Разбирает содержимое файла. Справочник нужен, чтобы узнать
    /// редкость предметов (в некоторых форматах её нет).
    fn parse(&self, content: &str, reference: &ReferenceData)
    -> Result<ParsedHistory, ImportError>;
}

/// Всё, что нужно для экспорта истории одной игры.
pub struct ExportInput<'a> {
    /// Молитвы в хронологическом порядке.
    pub wishes: &'a [Wish],
    /// Пулы баннеров игры (правила гаранта).
    pub pools: &'a [PoolDefinition],
    /// Справочник (расписание баннеров — для отметок «с баннера / нет»).
    pub reference: &'a ReferenceData,
    /// Профиль игрока (сервер и UID).
    pub profile: &'a GameProfile,
}

/// Ошибка экспорта.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("не удалось сформировать файл: {0}")]
pub struct ExportError(pub String);

/// Запись истории одной игры в файл одного формата.
pub trait HistoryExporter: Send + Sync {
    /// Идентификатор формата.
    fn format_id(&self) -> HistoryFormatId;

    /// Предлагаемое имя файла (пользователь может изменить его в диалоге).
    fn file_name(&self) -> String;

    /// Формирует содержимое файла.
    fn export(&self, input: &ExportInput<'_>) -> Result<String, ExportError>;
}
