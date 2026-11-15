//! Сервис профилей игр: сервер игрока.

use std::sync::Arc;

use domain::game::GameId;
use domain::region::ServerRegion;
use thiserror::Error;

use super::GameCatalog;
use crate::ports::{ProfileRepository, RepositoryError};

/// Ошибка сервиса профилей.
#[derive(Debug, Error)]
pub enum ProfileError {
    /// Игра не поддерживается: профиль для неё не создаётся.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Сохраняет настройки аккаунта в игре (пока — сервер игрока).
pub struct ProfileService {
    catalog: Arc<GameCatalog>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl ProfileService {
    /// Создаёт сервис поверх каталога игр и хранилища профилей.
    pub fn new(catalog: Arc<GameCatalog>, profile_repository: Arc<dyn ProfileRepository>) -> Self {
        Self {
            catalog,
            profile_repository,
        }
    }

    /// Сохраняет сервер игрока. Проверяет, что игра существует,
    /// чтобы в базу не попадали профили несуществующих игр.
    pub fn set_server_region(
        &self,
        game_id: &GameId,
        region: ServerRegion,
    ) -> Result<(), ProfileError> {
        if self.catalog.find(game_id).is_none() {
            return Err(ProfileError::UnknownGame(game_id.clone()));
        }
        Ok(self.profile_repository.set_server_region(game_id, region)?)
    }
}

// Тесты лежат рядом, в файле `profile_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "profile_service.tests.rs"]
mod tests;
