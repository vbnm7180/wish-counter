//! Модульные тесты для `export_service.rs`.
//!
//! Подключаются в `export_service.rs` строками
//! `#[cfg(test)] #[path = "export_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use domain::game::{GameInfo, GameModule};
use domain::import::HistoryExporter;
use domain::pity::rules::PoolDefinition;
use domain::profile::PlayerUid;
use domain::reference::ReferenceSnapshot;
use domain::region::ServerRegion;
use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, Wish, WishId, WishTime};

use super::*;

/// Формат тестового экспорта.
const FAKE_FORMAT: &str = "fake-format";

/// Тестовый экспорт: «файл» — число молитв, сервер и UID через `|`.
struct FakeExporter;

impl HistoryExporter for FakeExporter {
    fn format_id(&self) -> HistoryFormatId {
        HistoryFormatId::new(FAKE_FORMAT)
    }

    fn file_name(&self) -> String {
        "fake.json".into()
    }

    fn export(&self, input: &ExportInput<'_>) -> Result<String, ExportError> {
        Ok(format!(
            "{}|{:?}|{:?}",
            input.wishes.len(),
            input.profile.server_region,
            input.profile.uid.as_ref().map(PlayerUid::as_str)
        ))
    }
}

/// Тестовая игра с одним форматом экспорта.
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

    fn history_exporters(&self) -> Vec<Arc<dyn HistoryExporter>> {
        vec![Arc::new(FakeExporter)]
    }
}

/// Хранилище с заданными молитвами.
struct FixedWishes(Vec<Wish>);

impl WishRepository for FixedWishes {
    fn list_wishes(&self, _game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Ok(self.0.clone())
    }

    fn add_wishes(&self, _game_id: &GameId, _wishes: &[Wish]) -> Result<usize, RepositoryError> {
        unimplemented!("экспорт не записывает молитвы")
    }
    fn delete_wishes(&self, _game_id: &GameId) -> Result<(), RepositoryError> {
        unimplemented!("не используется в этом тесте")
    }
}

/// Профиль: Европа и UID.
struct FixedProfile;

impl ProfileRepository for FixedProfile {
    fn server_region(&self, _game_id: &GameId) -> Result<Option<ServerRegion>, RepositoryError> {
        Ok(Some(ServerRegion::Europe))
    }

    fn set_server_region(
        &self,
        _game_id: &GameId,
        _region: ServerRegion,
    ) -> Result<(), RepositoryError> {
        unimplemented!("экспорт не меняет профиль")
    }

    fn uid(&self, _game_id: &GameId) -> Result<Option<PlayerUid>, RepositoryError> {
        Ok(Some(PlayerUid::parse("700000000").unwrap()))
    }

    fn set_uid(&self, _game_id: &GameId, _uid: &PlayerUid) -> Result<(), RepositoryError> {
        unimplemented!("экспорт не меняет профиль")
    }
    fn delete_profile(&self, _game_id: &GameId) -> Result<(), RepositoryError> {
        unimplemented!("не используется в этом тесте")
    }
}

/// Сервис с тестовой игрой и `count` молитвами.
fn service(count: usize) -> ExportService {
    let wish = Wish {
        id: WishId::new("1"),
        banner_code: BannerCode::new(301),
        item_id: ItemId::new("sword"),
        item_kind: ItemKind::Weapon,
        rarity: Rarity::Three,
        time: WishTime::parse("2026-01-01 00:00:00").unwrap(),
    };
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    ExportService::new(
        catalog,
        references,
        Arc::new(FixedWishes(vec![wish; count])),
        Arc::new(FixedProfile),
    )
}

/// Экспорт тестовой игры в тестовый формат.
fn export(service: &ExportService) -> Result<ExportedFile, ExportServiceError> {
    service.export(&GameId::new("fake"), &HistoryFormatId::new(FAKE_FORMAT))
}

#[test]
fn passes_wishes_and_profile_to_exporter() {
    let file = export(&service(3)).unwrap();

    assert_eq!(file.file_name, "fake.json");
    assert_eq!(file.wish_count, 3);
    assert_eq!(file.content, "3|Some(Europe)|Some(\"700000000\")");
}

#[test]
fn empty_history_is_reported() {
    assert!(matches!(
        export(&service(0)),
        Err(ExportServiceError::EmptyHistory)
    ));
}

#[test]
fn unknown_format_and_game_are_reported() {
    let service = service(1);

    assert!(matches!(
        service.export(&GameId::new("fake"), &HistoryFormatId::new("uigf")),
        Err(ExportServiceError::UnsupportedFormat(_))
    ));
    assert!(matches!(
        service.export(&GameId::new("missing"), &HistoryFormatId::new(FAKE_FORMAT)),
        Err(ExportServiceError::UnknownGame(_))
    ));
}
