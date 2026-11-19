//! Импорт истории молитв из файла (Paimon.moe, позже UIGF).
//!
//! 1. найти игру и разборщик формата;
//! 2. разобрать файл (редкость предметов берётся из справочника);
//! 3. сохранить молитвы — уже сохранённые пропускаются, поэтому повторный
//!    импорт того же или дополненного файла безопасен;
//! 4. если в файле указаны сервер и UID — сохранить их в профиль игры.

use std::collections::BTreeSet;
use std::sync::Arc;

use domain::game::GameId;
use domain::import::{HistoryFormatId, ImportError};
use domain::profile::PlayerUid;
use domain::region::ServerRegion;
use domain::wish::ItemId;
use serde::Serialize;
use thiserror::Error;

use super::{GameCatalog, ReferenceService};
use crate::ports::{ProfileRepository, RepositoryError, WishRepository};

/// Максимальный размер файла истории. Реальные файлы — сотни килобайт;
/// ограничение защищает от случайного выбора огромного файла.
pub const MAX_IMPORT_BYTES: usize = 20 * 1024 * 1024;

/// Ошибка импорта.
#[derive(Debug, Error)]
pub enum ImportServiceError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Игра не умеет читать такой формат.
    #[error("формат «{}» не поддерживается", .0.as_str())]
    UnsupportedFormat(HistoryFormatId),
    /// Файл больше [`MAX_IMPORT_BYTES`].
    #[error("файл слишком большой")]
    FileTooLarge,
    /// Файл не удалось разобрать.
    #[error(transparent)]
    Parse(#[from] ImportError),
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Итог импорта для показа пользователю.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImportSummary {
    /// Сколько молитв прочитано из файла (без пропущенных повторов).
    pub total_in_file: usize,
    /// Сколько молитв добавлено.
    pub added: usize,
    /// Сколько молитв уже было сохранено раньше.
    pub already_present: usize,
    /// Сколько повторно записанных молитв пропущено (ошибка в самом файле).
    pub duplicates_skipped: usize,
    /// Сервер из файла (сохранён в профиль), если он был указан.
    pub server_region: Option<ServerRegion>,
    /// UID из файла (сохранён в профиль), если он был указан.
    pub uid: Option<PlayerUid>,
    /// Предметы из истории — для докачки их картинок. В JSON не передаётся.
    #[serde(skip)]
    pub item_ids: Vec<ItemId>,
}

/// Импортирует историю молитв из файлов.
pub struct ImportService {
    catalog: Arc<GameCatalog>,
    references: Arc<ReferenceService>,
    wish_repository: Arc<dyn WishRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl ImportService {
    /// Создаёт сервис импорта.
    pub fn new(
        catalog: Arc<GameCatalog>,
        references: Arc<ReferenceService>,
        wish_repository: Arc<dyn WishRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            catalog,
            references,
            wish_repository,
            profile_repository,
        }
    }

    /// Импортирует содержимое файла `content` в формате `format` для игры `game_id`.
    pub fn import(
        &self,
        game_id: &GameId,
        format: &HistoryFormatId,
        content: &str,
    ) -> Result<ImportSummary, ImportServiceError> {
        if content.len() > MAX_IMPORT_BYTES {
            return Err(ImportServiceError::FileTooLarge);
        }
        let module = self
            .catalog
            .find(game_id)
            .ok_or_else(|| ImportServiceError::UnknownGame(game_id.clone()))?;
        let parser = module
            .history_parsers()
            .into_iter()
            .find(|parser| &parser.format_id() == format)
            .ok_or_else(|| ImportServiceError::UnsupportedFormat(format.clone()))?;
        let reference = self
            .references
            .reference(game_id)
            .map_err(|_| ImportServiceError::UnknownGame(game_id.clone()))?;

        let history = parser.parse(content, &reference)?;
        let added = self.wish_repository.add_wishes(game_id, &history.wishes)?;
        if let Some(region) = history.server_region {
            self.profile_repository.set_server_region(game_id, region)?;
        }
        if let Some(uid) = &history.uid {
            self.profile_repository.set_uid(game_id, uid)?;
        }
        let summary = ImportSummary {
            total_in_file: history.wishes.len(),
            added,
            already_present: history.wishes.len() - added,
            duplicates_skipped: history.duplicates_skipped,
            server_region: history.server_region,
            uid: history.uid.clone(),
            item_ids: unique_items(&history.wishes),
        };
        tracing::info!(
            game = %game_id,
            format = format.as_str(),
            total = summary.total_in_file,
            added = summary.added,
            duplicates = summary.duplicates_skipped,
            "история импортирована"
        );
        Ok(summary)
    }
}

/// Уникальные предметы истории (отсортированы — для предсказуемого порядка докачки).
fn unique_items(wishes: &[domain::wish::Wish]) -> Vec<ItemId> {
    wishes
        .iter()
        .map(|wish| wish.item_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

// Тесты лежат рядом, в файле `import_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "import_service.tests.rs"]
mod tests;
