//! Модульные тесты для `profile_service.rs`.
//!
//! Подключаются в `profile_service.rs` строками
//! `#[cfg(test)] #[path = "profile_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::collections::HashMap;
use std::sync::Mutex;

use domain::game::{GameInfo, GameModule};
use domain::pity::rules::PoolDefinition;
use domain::reference::ReferenceSnapshot;

use super::*;

/// Тестовая игра «fake».
struct FakeGame;

impl GameModule for FakeGame {
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new("fake"),
            display_order: 1,
        }
    }

    fn pools(&self) -> Vec<PoolDefinition> {
        Vec::new()
    }

    fn embedded_reference(&self) -> ReferenceSnapshot {
        ReferenceSnapshot {
            source_revision: "test".into(),
            generated_at: "2026-01-01 00:00:00".into(),
            items: Vec::new(),
            banners: Vec::new(),
        }
    }
}

/// Хранилище профилей в памяти.
#[derive(Default)]
struct InMemoryProfiles(Mutex<HashMap<GameId, ServerRegion>>);

impl ProfileRepository for InMemoryProfiles {
    fn server_region(&self, game_id: &GameId) -> Result<Option<ServerRegion>, RepositoryError> {
        Ok(self.0.lock().unwrap().get(game_id).copied())
    }

    fn set_server_region(
        &self,
        game_id: &GameId,
        region: ServerRegion,
    ) -> Result<(), RepositoryError> {
        self.0.lock().unwrap().insert(game_id.clone(), region);
        Ok(())
    }
}

/// Сервис с тестовой игрой; возвращает и хранилище, чтобы проверить запись.
fn service() -> (ProfileService, Arc<InMemoryProfiles>) {
    let profiles = Arc::new(InMemoryProfiles::default());
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    (ProfileService::new(catalog, profiles.clone()), profiles)
}

#[test]
fn saves_region_for_known_game() {
    let (service, profiles) = service();

    service
        .set_server_region(&GameId::new("fake"), ServerRegion::Europe)
        .unwrap();

    assert_eq!(
        profiles.server_region(&GameId::new("fake")).unwrap(),
        Some(ServerRegion::Europe)
    );
}

#[test]
fn rejects_unknown_game() {
    let (service, profiles) = service();

    let result = service.set_server_region(&GameId::new("missing"), ServerRegion::Asia);

    assert!(matches!(result, Err(ProfileError::UnknownGame(_))));
    assert_eq!(
        profiles.server_region(&GameId::new("missing")).unwrap(),
        None
    );
}
