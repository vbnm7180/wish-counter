//! Модульные тесты для `history_service.rs`.
//!
//! Подключаются в `history_service.rs` строками
//! `#[cfg(test)] #[path = "history_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::sync::Mutex;

use domain::game::{GameInfo, GameModule};
use domain::pity::rules::{FiftyFiftyRule, PityRules, PoolDefinition, Probability};
use domain::profile::PlayerUid;
use domain::reference::{BannerInfo, ItemInfo, ReferenceSnapshot, ScheduleTime};
use domain::wish::BannerCode;

use super::*;

/// Код баннера персонажа в тестовой игре.
const CHARACTER_CODE: BannerCode = BannerCode::new(1);
/// Код стандартного баннера в тестовой игре.
const STANDARD_CODE: BannerCode = BannerCode::new(2);

/// Пул тестовой игры.
fn pool(id: &str, code: BannerCode, with_fifty_fifty: bool) -> PoolDefinition {
    PoolDefinition {
        id: PoolId::new(id),
        banner_codes: vec![code],
        rules: PityRules {
            five_star_hard_pity: 90,
            four_star_hard_pity: 10,
            fifty_fifty: with_fifty_fifty.then(|| FiftyFiftyRule {
                base_win_chance: Probability::from_basis_points(5_000),
                capturing_radiance: None,
            }),
            four_star_featured: None,
        },
        primogems_per_wish: 160,
    }
}

/// Тестовая игра: персонаж «hero» 5★ на баннере «Hero» (январь 2026).
struct FakeGame;

impl GameModule for FakeGame {
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new("fake"),
            display_order: 1,
        }
    }

    fn pools(&self) -> Vec<PoolDefinition> {
        vec![
            pool("character", CHARACTER_CODE, true),
            pool("standard", STANDARD_CODE, false),
        ]
    }

    fn embedded_reference(&self) -> ReferenceSnapshot {
        let at = |time: &str| ScheduleTime {
            time: WishTime::parse(time).unwrap(),
            asia_based: false,
        };
        ReferenceSnapshot {
            source_revision: "test".into(),
            generated_at: "2026-01-01 00:00:00".into(),
            items: vec![ItemInfo {
                id: ItemId::new("hero"),
                kind: ItemKind::Character,
                rarity: Rarity::Five,
                name: LocalizedText {
                    en: "Hero".into(),
                    ru: "Герой".into(),
                },
            }],
            banners: vec![BannerInfo {
                pool_id: PoolId::new("character"),
                name: "Hero Banner".into(),
                short_name: "Hero".into(),
                version: None,
                start: at("2026-01-01 00:00:00"),
                end: at("2026-01-21 00:00:00"),
                featured: vec![ItemId::new("hero")],
                featured_rare: vec![],
            }],
        }
    }
}

/// Хранилище с заданными молитвами.
struct FixedWishes(Vec<Wish>);

impl WishRepository for FixedWishes {
    fn list_wishes(&self, _game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Ok(self.0.clone())
    }

    fn add_wishes(&self, _game_id: &GameId, _wishes: &[Wish]) -> Result<usize, RepositoryError> {
        unimplemented!("история не записывает молитвы")
    }
}

/// Профиль с заданным сервером.
struct FixedRegion(Mutex<Option<ServerRegion>>);

impl ProfileRepository for FixedRegion {
    fn server_region(&self, _game_id: &GameId) -> Result<Option<ServerRegion>, RepositoryError> {
        Ok(*self.0.lock().unwrap())
    }

    fn set_server_region(
        &self,
        _game_id: &GameId,
        _region: ServerRegion,
    ) -> Result<(), RepositoryError> {
        unimplemented!("история не меняет профиль")
    }

    fn uid(&self, _game_id: &GameId) -> Result<Option<PlayerUid>, RepositoryError> {
        Ok(None)
    }

    fn set_uid(&self, _game_id: &GameId, _uid: &PlayerUid) -> Result<(), RepositoryError> {
        unimplemented!("история не меняет профиль")
    }
}

/// Молитва номер `index` (каждая следующая на минуту позже).
fn wish(index: u32, code: BannerCode, item: &str, rarity: Rarity) -> Wish {
    Wish {
        id: WishId::new(index.to_string()),
        banner_code: code,
        item_id: ItemId::new(item),
        item_kind: ItemKind::Character,
        rarity,
        time: WishTime::parse(format!("2026-01-10 12:{index:02}:00")).unwrap(),
    }
}

/// Сервис: на баннере персонажа 3★ и затем «hero» 5★, на стандартном — одна 3★.
fn service(region: Option<ServerRegion>) -> HistoryService {
    let wishes = vec![
        wish(1, CHARACTER_CODE, "sword", Rarity::Three),
        wish(2, STANDARD_CODE, "sword", Rarity::Three),
        wish(3, CHARACTER_CODE, "hero", Rarity::Five),
    ];
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    HistoryService::new(
        catalog,
        references,
        Arc::new(FixedWishes(wishes)),
        Arc::new(FixedRegion(Mutex::new(region))),
    )
}

/// История пула персонажа.
fn character_history(service: &HistoryService) -> PoolHistory {
    service
        .pool_history(&GameId::new("fake"), &PoolId::new("character"))
        .unwrap()
}

#[test]
fn rows_belong_to_pool_and_newest_come_first() {
    let history = character_history(&service(Some(ServerRegion::Asia)));

    let ids: Vec<&str> = history.rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids, ["3", "1"]);
}

#[test]
fn rows_contain_pity_outcome_name_and_banner() {
    let history = character_history(&service(Some(ServerRegion::Asia)));
    let hero = &history.rows[0];

    assert_eq!(hero.pity, Some(2));
    assert_eq!(hero.outcome, Some(FiftyFiftyOutcome::Won));
    assert_eq!(hero.name.as_ref().unwrap().ru, "Герой");
    assert_eq!(hero.banner_name.as_deref(), Some("Hero"));
    // 3★: без номера по гаранту и без исхода; предмета нет в справочнике — без названия.
    assert_eq!(history.rows[1].pity, None);
    assert_eq!(history.rows[1].outcome, None);
    assert_eq!(history.rows[1].name, None);
}

/// Без сервера исход 50/50 неизвестен, название баннера не определяется.
#[test]
fn without_region_outcome_and_banner_are_unknown() {
    let history = character_history(&service(None));
    let hero = &history.rows[0];

    assert_eq!(history.server_region, None);
    assert_eq!(hero.outcome, Some(FiftyFiftyOutcome::Unknown));
    assert_eq!(hero.banner_name, None);
    assert_eq!(hero.pity, Some(2), "номер по гаранту не зависит от сервера");
}

#[test]
fn unknown_pool_and_game_are_reported() {
    let service = service(None);

    assert!(matches!(
        service.pool_history(&GameId::new("fake"), &PoolId::new("missing")),
        Err(HistoryError::UnknownPool(_))
    ));
    assert!(matches!(
        service.pool_history(&GameId::new("missing"), &PoolId::new("character")),
        Err(HistoryError::UnknownGame(_))
    ));
}
