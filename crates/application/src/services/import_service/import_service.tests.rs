//! Модульные тесты для `import_service.rs`.
//!
//! Подключаются в `import_service.rs` строками
//! `#[cfg(test)] #[path = "import_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::collections::HashMap;
use std::sync::Mutex;

use domain::game::{GameInfo, GameModule};
use domain::import::{HistoryParser, ParsedHistory};
use domain::pity::rules::PoolDefinition;
use domain::reference::{ReferenceData, ReferenceSnapshot};
use domain::wish::{BannerCode, ItemKind, Rarity, Wish, WishId, WishTime};

use domain::profile::PlayerUid;

use super::*;

/// Формат тестового разборщика.
const FAKE_FORMAT: &str = "fake-format";

/// Тестовый разборщик. Содержимое файла:
/// - `"<число>"` — столько молитв (предметы `item_0`, `item_1`, `item_0`, ...);
/// - `"<число>!"` — то же, сервер «Европа» и UID 700000000;
/// - `"bad"` — ошибка разбора.
struct FakeParser;

impl HistoryParser for FakeParser {
    fn format_id(&self) -> HistoryFormatId {
        HistoryFormatId::new(FAKE_FORMAT)
    }

    fn parse(
        &self,
        content: &str,
        _reference: &ReferenceData,
    ) -> Result<ParsedHistory, ImportError> {
        let with_region = content.ends_with('!');
        let count: usize =
            content
                .trim_end_matches('!')
                .parse()
                .map_err(|_| ImportError::InvalidFile {
                    format: FAKE_FORMAT.into(),
                    reason: "не число".into(),
                })?;
        Ok(ParsedHistory {
            wishes: (0..count).map(wish).collect(),
            server_region: with_region.then_some(ServerRegion::Europe),
            uid: with_region.then(|| PlayerUid::parse("700000000").unwrap()),
            duplicates_skipped: 1,
        })
    }
}

/// Молитва номер `index`.
fn wish(index: usize) -> Wish {
    Wish {
        id: WishId::new(format!("w{index}")),
        banner_code: BannerCode::new(301),
        item_id: ItemId::new(format!("item_{}", index % 2)),
        item_kind: ItemKind::Weapon,
        rarity: Rarity::Three,
        time: WishTime::parse("2026-01-01 00:00:00").unwrap(),
    }
}

/// Тестовая игра с одним форматом импорта.
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

    fn history_parsers(&self) -> Vec<Arc<dyn HistoryParser>> {
        vec![Arc::new(FakeParser)]
    }
}

/// Хранилище молитв в памяти: пропускает молитвы с уже известным id.
#[derive(Default)]
struct InMemoryWishes(Mutex<Vec<Wish>>);

impl WishRepository for InMemoryWishes {
    fn list_wishes(&self, _game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Ok(self.0.lock().unwrap().clone())
    }

    fn add_wishes(&self, _game_id: &GameId, wishes: &[Wish]) -> Result<usize, RepositoryError> {
        let mut stored = self.0.lock().unwrap();
        let mut added = 0;
        for wish in wishes {
            if !stored.iter().any(|existing| existing.id == wish.id) {
                stored.push(wish.clone());
                added += 1;
            }
        }
        Ok(added)
    }

    fn delete_wishes(&self, _game_id: &GameId) -> Result<(), RepositoryError> {
        Ok(())
    }
}

/// Профили в памяти.
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

/// Сервис и доступ к хранилищам.
struct Setup {
    service: ImportService,
    profiles: Arc<InMemoryProfiles>,
}

/// Собирает сервис с тестовой игрой.
fn setup() -> Setup {
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    let profiles = Arc::new(InMemoryProfiles::default());
    let service = ImportService::new(
        catalog,
        references,
        Arc::new(InMemoryWishes::default()),
        profiles.clone(),
    );
    Setup { service, profiles }
}

/// Импортирует содержимое в тестовую игру.
fn import(setup: &Setup, content: &str) -> Result<ImportSummary, ImportServiceError> {
    setup.service.import(
        &GameId::new("fake"),
        &HistoryFormatId::new(FAKE_FORMAT),
        content,
    )
}

#[test]
fn imports_wishes_and_reports_counts() {
    let setup = setup();

    let summary = import(&setup, "5").unwrap();

    assert_eq!(summary.total_in_file, 5);
    assert_eq!(summary.added, 5);
    assert_eq!(summary.already_present, 0);
    assert_eq!(summary.duplicates_skipped, 1);
}

/// Повторный импорт дополненного файла добавляет только новые молитвы.
#[test]
fn reimport_adds_only_new_wishes() {
    let setup = setup();
    import(&setup, "5").unwrap();

    let summary = import(&setup, "7").unwrap();

    assert_eq!(summary.added, 2);
    assert_eq!(summary.already_present, 5);
}

#[test]
fn server_region_from_file_is_saved() {
    let setup = setup();

    let summary = import(&setup, "1!").unwrap();

    assert_eq!(summary.server_region, Some(ServerRegion::Europe));
    assert_eq!(
        setup.profiles.server_region(&GameId::new("fake")).unwrap(),
        Some(ServerRegion::Europe)
    );
    assert_eq!(
        setup
            .profiles
            .uid(&GameId::new("fake"))
            .unwrap()
            .unwrap()
            .as_str(),
        "700000000"
    );
}

/// Без сервера в файле выбранный ранее сервер не сбрасывается.
#[test]
fn missing_region_keeps_previous_choice() {
    let setup = setup();
    setup
        .profiles
        .set_server_region(&GameId::new("fake"), ServerRegion::Asia)
        .unwrap();

    import(&setup, "1").unwrap();

    assert_eq!(
        setup.profiles.server_region(&GameId::new("fake")).unwrap(),
        Some(ServerRegion::Asia)
    );
}

#[test]
fn item_ids_are_unique() {
    let summary = import(&setup(), "5").unwrap();
    let ids: Vec<&str> = summary.item_ids.iter().map(ItemId::as_str).collect();
    assert_eq!(ids, ["item_0", "item_1"]);
}

#[test]
fn errors_are_reported() {
    let setup = setup();

    assert!(matches!(
        import(&setup, "bad"),
        Err(ImportServiceError::Parse(_))
    ));
    assert!(matches!(
        setup
            .service
            .import(&GameId::new("fake"), &HistoryFormatId::new("uigf"), "1"),
        Err(ImportServiceError::UnsupportedFormat(_))
    ));
    assert!(matches!(
        setup.service.import(
            &GameId::new("missing"),
            &HistoryFormatId::new(FAKE_FORMAT),
            "1"
        ),
        Err(ImportServiceError::UnknownGame(_))
    ));
}

#[test]
fn oversized_file_is_rejected_before_parsing() {
    let huge = "1".repeat(MAX_IMPORT_BYTES + 1);
    assert!(matches!(
        import(&setup(), &huge),
        Err(ImportServiceError::FileTooLarge)
    ));
}
