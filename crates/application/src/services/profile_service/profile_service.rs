//! Сервис профилей игр: сервер игрока.

use std::sync::Arc;

use domain::game::GameId;
use domain::profile::{GameProfile, PlayerUid};
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

/// Читает и сохраняет профиль игрока в игре: сервер и UID.
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

    /// Возвращает сервер игрока; `None`, если ещё не выбран.
    pub fn server_region(&self, game_id: &GameId) -> Result<Option<ServerRegion>, ProfileError> {
        self.ensure_known(game_id)?;
        Ok(self.profile_repository.server_region(game_id)?)
    }

    /// Возвращает профиль игрока целиком.
    pub fn profile(&self, game_id: &GameId) -> Result<GameProfile, ProfileError> {
        self.ensure_known(game_id)?;
        Ok(GameProfile {
            server_region: self.profile_repository.server_region(game_id)?,
            uid: self.profile_repository.uid(game_id)?,
        })
    }

    /// Сохраняет UID игрока.
    pub fn set_uid(&self, game_id: &GameId, uid: &PlayerUid) -> Result<(), ProfileError> {
        self.ensure_known(game_id)?;
        Ok(self.profile_repository.set_uid(game_id, uid)?)
    }

    /// Сохраняет сервер игрока. Проверяет, что игра существует,
    /// чтобы в базу не попадали профили несуществующих игр.
    pub fn set_server_region(
        &self,
        game_id: &GameId,
        region: ServerRegion,
    ) -> Result<(), ProfileError> {
        self.ensure_known(game_id)?;
        Ok(self.profile_repository.set_server_region(game_id, region)?)
    }

    /// Проверяет, что игра зарегистрирована.
    fn ensure_known(&self, game_id: &GameId) -> Result<(), ProfileError> {
        match self.catalog.find(game_id) {
            Some(_) => Ok(()),
            None => Err(ProfileError::UnknownGame(game_id.clone())),
        }
    }
}

// Тесты лежат рядом, в файле `profile_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "profile_service.tests.rs"]
mod tests;
