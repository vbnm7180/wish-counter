//! Экспорт истории молитв в файл другого сервиса (сейчас — Paimon.moe).
//!
//! Сервис только формирует содержимое файла; куда его сохранить, решает
//! пользователь в диалоге (это делает слой приложения Tauri).

use std::sync::Arc;

use domain::game::GameId;
use domain::import::{ExportError, ExportInput, HistoryFormatId};
use domain::profile::GameProfile;
use thiserror::Error;

use super::{GameCatalog, ReferenceService};
use crate::ports::{ProfileRepository, RepositoryError, WishRepository};

/// Ошибка экспорта.
#[derive(Debug, Error)]
pub enum ExportServiceError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Игра не умеет писать такой формат.
    #[error("формат «{}» не поддерживается", .0.as_str())]
    UnsupportedFormat(HistoryFormatId),
    /// История пуста — экспортировать нечего.
    #[error("история молитв пуста")]
    EmptyHistory,
    /// Не удалось сформировать файл.
    #[error(transparent)]
    Export(#[from] ExportError),
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Готовый к сохранению файл.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedFile {
    /// Предлагаемое имя файла.
    pub file_name: String,
    /// Содержимое файла.
    pub content: String,
    /// Сколько молитв в файле.
    pub wish_count: usize,
}

/// Формирует файлы экспорта истории.
pub struct ExportService {
    catalog: Arc<GameCatalog>,
    references: Arc<ReferenceService>,
    wish_repository: Arc<dyn WishRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl ExportService {
    /// Создаёт сервис экспорта.
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

    /// Формирует файл истории игры `game_id` в формате `format`.
    pub fn export(
        &self,
        game_id: &GameId,
        format: &HistoryFormatId,
    ) -> Result<ExportedFile, ExportServiceError> {
        let module = self
            .catalog
            .find(game_id)
            .ok_or_else(|| ExportServiceError::UnknownGame(game_id.clone()))?;
        let exporter = module
            .history_exporters()
            .into_iter()
            .find(|exporter| &exporter.format_id() == format)
            .ok_or_else(|| ExportServiceError::UnsupportedFormat(format.clone()))?;
        let wishes = self.wish_repository.list_wishes(game_id)?;
        if wishes.is_empty() {
            return Err(ExportServiceError::EmptyHistory);
        }
        let reference = self
            .references
            .reference(game_id)
            .map_err(|_| ExportServiceError::UnknownGame(game_id.clone()))?;
        let profile = GameProfile {
            server_region: self.profile_repository.server_region(game_id)?,
            uid: self.profile_repository.uid(game_id)?,
        };
        let pools = module.pools();
        let content = exporter.export(&ExportInput {
            wishes: &wishes,
            pools: &pools,
            reference: &reference,
            profile: &profile,
        })?;
        tracing::info!(game = %game_id, format = format.as_str(), wishes = wishes.len(), "история экспортирована");
        Ok(ExportedFile {
            file_name: exporter.file_name(),
            content,
            wish_count: wishes.len(),
        })
    }
}

// Тесты лежат рядом, в файле `export_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "export_service.tests.rs"]
mod tests;
