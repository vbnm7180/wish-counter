//! Модульные тесты для `statistics_service.rs`.
//!
//! Подключаются в `statistics_service.rs` строками
//! `#[cfg(test)] #[path = "statistics_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::sync::Mutex;

use domain::game::GameInfo;
use domain::pity::rules::{FiftyFiftyRule, PityRules, Probability};
use domain::pity::statistics::FeaturedStatus;
use domain::reference::{BannerInfo, ReferenceSnapshot, ScheduleTime};
use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, WishId, WishTime};

use super::*;

/// Код «баннера персонажа» в тестовой игре.
const CHARACTER_CODE: BannerCode = BannerCode::new(1);
/// Код «стандартного баннера» в тестовой игре.
const STANDARD_CODE: BannerCode = BannerCode::new(2);
/// Код баннера, которого нет ни в одном пуле.
const UNKNOWN_CODE: BannerCode = BannerCode::new(99);
/// Время всех тестовых молитв — внутри единственного баннера персонажа.
const WISH_TIME: &str = "2026-01-10 12:00:00";
/// Предмет с баннера персонажа.
const FEATURED_ITEM: &str = "featured_hero";

/// Тестовая игра: пулы «персонажи» (с 50/50) и «стандартный»,
/// в справочнике один баннер персонажа на январь 2026.
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
            fake_pool("character", CHARACTER_CODE, true),
            fake_pool("standard", STANDARD_CODE, false),
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
            items: Vec::new(),
            banners: vec![BannerInfo {
                pool_id: PoolId::new("character"),
                name: "Test".into(),
                short_name: "Test".into(),
                version: None,
                start: at("2026-01-01 00:00:00"),
                end: at("2026-01-21 00:00:00"),
                featured: vec![ItemId::new(FEATURED_ITEM)],
                featured_rare: vec![],
            }],
        }
    }
}

/// Пул тестовой игры: гарант 90/10, 160 примогемов за молитву.
fn fake_pool(id: &str, code: BannerCode, with_fifty_fifty: bool) -> PoolDefinition {
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

/// Хранилище, которое возвращает заданный список молитв для любой игры.
struct FixedWishes(Vec<Wish>);

impl WishRepository for FixedWishes {
    fn list_wishes(&self, _game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Ok(self.0.clone())
    }

    fn add_wishes(&self, _game_id: &GameId, _wishes: &[Wish]) -> Result<usize, RepositoryError> {
        unimplemented!("сервис статистики не записывает молитвы")
    }
}

/// Хранилище, которое всегда возвращает ошибку чтения.
struct BrokenRepository;

impl WishRepository for BrokenRepository {
    fn list_wishes(&self, _game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Err(RepositoryError::Io("disk".into()))
    }

    fn add_wishes(&self, _game_id: &GameId, _wishes: &[Wish]) -> Result<usize, RepositoryError> {
        unimplemented!("сервис статистики не записывает молитвы")
    }
}

/// Профиль с заранее заданным сервером (или без него).
struct FixedRegion(Mutex<Option<ServerRegion>>);

impl ProfileRepository for FixedRegion {
    fn server_region(&self, _game_id: &GameId) -> Result<Option<ServerRegion>, RepositoryError> {
        Ok(*self.0.lock().unwrap())
    }

    fn set_server_region(
        &self,
        _game_id: &GameId,
        region: ServerRegion,
    ) -> Result<(), RepositoryError> {
        *self.0.lock().unwrap() = Some(region);
        Ok(())
    }
}

/// Создаёт молитву на указанном баннере.
fn wish(code: BannerCode, rarity: Rarity, item: &str) -> Wish {
    Wish {
        id: WishId::new("1"),
        banner_code: code,
        item_id: ItemId::new(item),
        item_kind: ItemKind::Character,
        rarity,
        time: WishTime::parse(WISH_TIME).unwrap(),
    }
}

/// Создаёт `count` молитв с 3★ на указанном баннере.
fn wishes_on(code: BannerCode, count: usize) -> Vec<Wish> {
    vec![wish(code, Rarity::Three, "sword"); count]
}

/// Создаёт сервис с тестовой игрой, хранилищем и сервером игрока.
fn service_with(
    repository: impl WishRepository + 'static,
    region: Option<ServerRegion>,
) -> StatisticsService {
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    let profiles = Arc::new(FixedRegion(Mutex::new(region)));
    StatisticsService::new(catalog, Arc::new(repository), references, profiles)
}

/// Сервис без выбранного сервера.
fn service(repository: impl WishRepository + 'static) -> StatisticsService {
    service_with(repository, None)
}

/// Запрашивает статистику тестовой игры.
fn fake_statistics(service: &StatisticsService) -> GameStatistics {
    service.game_statistics(&GameId::new("fake")).unwrap()
}

#[test]
fn splits_wishes_by_pool_and_sums_totals() {
    let wishes = [wishes_on(CHARACTER_CODE, 12), wishes_on(STANDARD_CODE, 3)].concat();

    let statistics = fake_statistics(&service(FixedWishes(wishes)));

    assert_eq!(statistics.pools[0].statistics.total_wishes, 12);
    assert_eq!(statistics.pools[0].primogems_spent, 1_920);
    assert_eq!(statistics.pools[1].statistics.total_wishes, 3);
    assert_eq!(statistics.total_wishes, 15);
    assert_eq!(statistics.primogems_spent, 2_400);
}

#[test]
fn ignores_wishes_from_unknown_banners() {
    let statistics = fake_statistics(&service(FixedWishes(wishes_on(UNKNOWN_CODE, 5))));
    assert_eq!(statistics.total_wishes, 0);
}

#[test]
fn keeps_pool_order_from_game_module() {
    let statistics = fake_statistics(&service(FixedWishes(vec![])));
    let ids: Vec<&str> = statistics
        .pools
        .iter()
        .map(|pool| pool.pool_id.as_str())
        .collect();
    assert_eq!(ids, ["character", "standard"]);
}

#[test]
fn featured_status_present_only_for_pools_with_fifty_fifty() {
    let statistics = fake_statistics(&service(FixedWishes(vec![])));

    assert!(matches!(
        statistics.pools[0].statistics.featured,
        Some(FeaturedStatus::Known { .. })
    ));
    assert_eq!(statistics.pools[1].statistics.featured, None);
}

/// Без выбранного сервера исход 50/50 не определяется.
#[test]
fn fifty_fifty_is_unknown_without_server_region() {
    let wishes = vec![wish(CHARACTER_CODE, Rarity::Five, FEATURED_ITEM)];

    let statistics = fake_statistics(&service(FixedWishes(wishes)));

    assert_eq!(statistics.server_region, None);
    assert_eq!(
        statistics.pools[0].statistics.featured,
        Some(FeaturedStatus::Unknown)
    );
}

/// С выбранным сервером исход 50/50 определяется по расписанию баннеров.
#[test]
fn fifty_fifty_is_resolved_with_server_region() {
    let wishes = vec![wish(CHARACTER_CODE, Rarity::Five, "standard_hero")];

    let statistics = fake_statistics(&service_with(
        FixedWishes(wishes),
        Some(ServerRegion::Europe),
    ));

    assert_eq!(statistics.server_region, Some(ServerRegion::Europe));
    match statistics.pools[0].statistics.featured {
        Some(FeaturedStatus::Known { guaranteed, .. }) => {
            assert!(guaranteed, "после проигрыша следующий 5★ гарантирован")
        }
        other => panic!("ожидался известный статус, получено {other:?}"),
    }
}

#[test]
fn unknown_game_is_reported() {
    let result = service(FixedWishes(vec![])).game_statistics(&GameId::new("missing"));
    assert!(matches!(result, Err(StatisticsError::UnknownGame(_))));
}

#[test]
fn repository_errors_are_propagated() {
    let result = service(BrokenRepository).game_statistics(&GameId::new("fake"));
    assert!(matches!(result, Err(StatisticsError::Repository(_))));
}

/// Поля пула «разворачиваются» на верхний уровень JSON (serde flatten).
#[test]
fn pool_summary_serializes_flat() {
    let statistics = fake_statistics(&service(FixedWishes(vec![])));

    let json = serde_json::to_string(&statistics.pools[1]).unwrap();

    assert!(
        json.starts_with(r#"{"pool_id":"standard","primogems_spent":0,"total_wishes":0"#),
        "{json}"
    );
}
