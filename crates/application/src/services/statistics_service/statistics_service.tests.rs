//! Модульные тесты для `statistics_service.rs`.
//!
//! Подключаются в `statistics_service.rs` строками
//! `#[cfg(test)] #[path = "statistics_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use domain::game::GameInfo;
use domain::pity::featured::NoFeaturedData;
use domain::pity::rules::PityRules;
use domain::pity::statistics::FeaturedStatus;
use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, WishId, WishTime};

use super::*;

/// Код «баннера персонажа» в тестовой игре.
const CHARACTER_CODE: BannerCode = BannerCode::new(1);
/// Код «стандартного баннера» в тестовой игре.
const STANDARD_CODE: BannerCode = BannerCode::new(2);
/// Код баннера, которого нет ни в одном пуле.
const UNKNOWN_CODE: BannerCode = BannerCode::new(99);

/// Тестовая игра с двумя пулами: «персонажи» (с 50/50) и «стандартный».
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

    fn featured_lookup(&self) -> Arc<dyn FeaturedLookup> {
        Arc::new(NoFeaturedData)
    }
}

/// Пул тестовой игры: гарант 90/10, 160 примогемов за молитву.
fn fake_pool(id: &str, code: BannerCode, with_fifty_fifty: bool) -> PoolDefinition {
    use domain::pity::rules::{FiftyFiftyRule, Probability};
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
}

/// Хранилище, которое всегда возвращает ошибку чтения.
struct BrokenRepository;

impl WishRepository for BrokenRepository {
    fn list_wishes(&self, _game_id: &GameId) -> Result<Vec<Wish>, RepositoryError> {
        Err(RepositoryError::Io("disk".into()))
    }
}

/// Создаёт `count` молитв с 3★ на указанном баннере.
fn wishes_on(code: BannerCode, count: usize) -> Vec<Wish> {
    let wish = Wish {
        id: WishId::new("1"),
        banner_code: code,
        item_id: ItemId::new("sword"),
        item_kind: ItemKind::Weapon,
        rarity: Rarity::Three,
        time: WishTime::parse("2026-01-01 00:00:00").unwrap(),
    };
    vec![wish; count]
}

/// Создаёт сервис с тестовой игрой и заданным хранилищем.
fn service(repository: impl WishRepository + 'static) -> StatisticsService {
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    StatisticsService::new(catalog, Arc::new(repository))
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
