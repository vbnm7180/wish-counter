//! Очистка данных игры для смены аккаунта.
//!
//! Удаляет все молитвы и профиль (сервер, UID) выбранной игры.
//! Фоны приложения и настройки не затрагиваются.

use std::sync::Arc;

use domain::game::GameId;
use serde::Serialize;
use thiserror::Error;

use super::GameCatalog;
use crate::ports::{ProfileRepository, RepositoryError, WishRepository};

/// Ошибка очистки данных игры.
#[derive(Debug, Error)]
pub enum ClearGameError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Итог очистки для показа пользователю.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClearSummary {
    /// Имя игры, данные которой удалены.
    pub game_id: String,
}

/// Удаляет все данные игры (молитвы и профиль) для смены аккаунта.
pub struct ClearGameService {
    catalog: Arc<GameCatalog>,
    wish_repository: Arc<dyn WishRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl ClearGameService {
    /// Создаёт сервис очистки.
    pub fn new(
        catalog: Arc<GameCatalog>,
        wish_repository: Arc<dyn WishRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            catalog,
            wish_repository,
            profile_repository,
        }
    }

    /// Удаляет все молитвы и профиль игры.
    pub fn clear(&self, game_id: &GameId) -> Result<ClearSummary, ClearGameError> {
        // Проверяем, что игра существует.
        self.catalog
            .find(game_id)
            .ok_or_else(|| ClearGameError::UnknownGame(game_id.clone()))?;

        self.wish_repository.delete_wishes(game_id)?;
        self.profile_repository.delete_profile(game_id)?;

        tracing::info!(game = %game_id, "данные игры очищены");

        Ok(ClearSummary {
            game_id: game_id.as_str().to_string(),
        })
    }
}
