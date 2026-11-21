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

use domain::profile::PlayerUid;

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
/// Второе поле — UID игроков.
struct InMemoryProfiles(
    Mutex<HashMap<GameId, ServerRegion>>,
    Mutex<HashMap<GameId, PlayerUid>>,
);

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

    fn uid(&self, game_id: &GameId) -> Result<Option<PlayerUid>, RepositoryError> {
        Ok(self.1.lock().unwrap().get(game_id).cloned())
    }

    fn set_uid(&self, game_id: &GameId, uid: &PlayerUid) -> Result<(), RepositoryError> {
        self.1.lock().unwrap().insert(game_id.clone(), uid.clone());
        Ok(())
    }

    fn delete_profile(&self, _game_id: &GameId) -> Result<(), RepositoryError> {
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

#[test]
fn reads_saved_region() {
    let (service, _profiles) = service();
    let fake = GameId::new("fake");

    assert_eq!(service.server_region(&fake).unwrap(), None);
    service
        .set_server_region(&fake, ServerRegion::America)
        .unwrap();

    assert_eq!(
        service.server_region(&fake).unwrap(),
        Some(ServerRegion::America)
    );
    assert!(service.server_region(&GameId::new("missing")).is_err());
}

#[test]
fn profile_contains_region_and_uid() {
    let (service, _profiles) = service();
    let fake = GameId::new("fake");
    let uid = PlayerUid::parse("700000000").unwrap();

    service
        .set_server_region(&fake, ServerRegion::Asia)
        .unwrap();
    service.set_uid(&fake, &uid).unwrap();

    let profile = service.profile(&fake).unwrap();
    assert_eq!(profile.server_region, Some(ServerRegion::Asia));
    assert_eq!(profile.uid, Some(uid));
}
