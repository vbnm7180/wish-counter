//! Сквозной тест: SQLite → StatisticsService → правила и справочник Genshin.
//!
//! В отличие от модульных тестов (рядом с кодом), этот файл лежит в папке
//! `tests/` крейта: Cargo собирает его как отдельную программу, которая
//! видит только публичный API. Здесь проверяется, что слои правильно
//! стыкуются друг с другом — так же, как в `bootstrap.rs`, — на **настоящем**
//! встроенном справочнике (баннер Mitya, версия 7.2).

use std::sync::Arc;

use application::ports::{ProfileRepository, WishRepository};
use application::services::{GameCatalog, PoolSummary, ReferenceService, StatisticsService};
use domain::game::GameId;
use domain::pity::statistics::{ChanceSource, FeaturedStatus};
use domain::region::ServerRegion;
use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, Wish, WishId, WishTime};
use game_genshin::{GENSHIN_GAME_ID, GenshinModule, banner_codes, pool_ids};
use infrastructure::database::{SharedConnection, open_in_memory_database};
use infrastructure::profiles::SqliteProfileRepository;
use infrastructure::wishes::SqliteWishRepository;

/// Начало серии молитв — середина баннера Mitya (7.2).
const FIRST_WISH_TIME: &str = "2026-11-10 12:00:00";

/// Собранные вместе хранилища и сервис, как в `bootstrap.rs`.
struct TestApp {
    wishes: Arc<SqliteWishRepository>,
    profiles: Arc<SqliteProfileRepository>,
    statistics: StatisticsService,
}

/// Собирает приложение на базе SQLite в памяти.
fn test_app() -> TestApp {
    let database = SharedConnection::new(open_in_memory_database().unwrap());
    let wishes = Arc::new(SqliteWishRepository::new(database.clone()));
    let profiles = Arc::new(SqliteProfileRepository::new(database));
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(GenshinModule)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    let statistics = StatisticsService::new(catalog, wishes.clone(), references, profiles.clone());
    TestApp {
        wishes,
        profiles,
        statistics,
    }
}

/// Молитва номер `index` из серии: каждая следующая на секунду позже.
fn wish(index: i64, banner: BannerCode, rarity: Rarity, item: &str) -> Wish {
    let start = WishTime::parse(FIRST_WISH_TIME).unwrap().to_date_time();
    Wish {
        id: WishId::new(format!("{index:019}")),
        banner_code: banner,
        item_id: ItemId::new(item),
        item_kind: ItemKind::Character,
        rarity,
        time: WishTime::from_date_time(start + chrono::Duration::seconds(index)),
    }
}

/// История на баннерах персонажа (301 и 400 чередуются — общий счётчик):
/// 4★ не с баннера (the_flute) на 5-й, 5★ Дилюк (проигрыш 50/50) на 75-й,
/// 5★ Митя (по гаранту) на 85-й, затем ещё 5 молитв; плюс 3 молитвы на оружейном.
fn history() -> Vec<Wish> {
    let character_wishes = (1..=90).map(|index| {
        let banner = if index % 2 == 0 {
            banner_codes::CHARACTER
        } else {
            banner_codes::CHARACTER_2
        };
        match index {
            5 => wish(index, banner, Rarity::Four, "the_flute"),
            75 => wish(index, banner, Rarity::Five, "diluc"),
            85 => wish(index, banner, Rarity::Five, "mitya"),
            _ => wish(index, banner, Rarity::Three, "cool_steel"),
        }
    });
    let weapon_wishes =
        (91..=93).map(|index| wish(index, banner_codes::WEAPON, Rarity::Three, "cool_steel"));
    character_wishes.chain(weapon_wishes).collect()
}

/// Находит карточку пула по идентификатору.
fn pool<'a>(pools: &'a [PoolSummary], id: &str) -> &'a PoolSummary {
    pools
        .iter()
        .find(|pool| pool.pool_id.as_str() == id)
        .unwrap()
}

#[test]
fn counters_and_primogems_are_calculated_from_sqlite_history() {
    let app = test_app();
    let genshin = GameId::new(GENSHIN_GAME_ID);
    app.wishes.add_wishes(&genshin, &history()).unwrap();

    let statistics = app.statistics.game_statistics(&genshin).unwrap();

    let character = pool(&statistics.pools, pool_ids::CHARACTER);
    assert_eq!(character.statistics.total_wishes, 90);
    assert_eq!(character.statistics.five_star.current, 5);
    assert_eq!(character.statistics.five_star.remaining, 85);
    assert_eq!(character.primogems_spent, 90 * 160);
    assert_eq!(
        pool(&statistics.pools, pool_ids::WEAPON)
            .statistics
            .total_wishes,
        3
    );
    assert_eq!(statistics.total_wishes, 93);
    assert_eq!(statistics.primogems_spent, 93 * 160);
}

/// Без выбранного сервера исход 50/50 неизвестен.
#[test]
fn fifty_fifty_is_unknown_until_server_is_chosen() {
    let app = test_app();
    let genshin = GameId::new(GENSHIN_GAME_ID);
    app.wishes.add_wishes(&genshin, &history()).unwrap();

    let statistics = app.statistics.game_statistics(&genshin).unwrap();

    assert_eq!(statistics.server_region, None);
    assert_eq!(
        pool(&statistics.pools, pool_ids::CHARACTER)
            .statistics
            .featured,
        Some(FeaturedStatus::Unknown)
    );
}

/// С выбранным сервером исходы определяются по настоящему расписанию баннеров:
/// Дилюк — проигрыш 50/50, Митя — гарант; the_flute — проигрыш для 4★.
#[test]
fn fifty_fifty_is_resolved_with_real_banner_schedule() {
    let app = test_app();
    let genshin = GameId::new(GENSHIN_GAME_ID);
    app.wishes.add_wishes(&genshin, &history()).unwrap();
    app.profiles
        .set_server_region(&genshin, ServerRegion::Europe)
        .unwrap();

    let statistics = app.statistics.game_statistics(&genshin).unwrap();
    let character = &pool(&statistics.pools, pool_ids::CHARACTER).statistics;

    match character.featured {
        Some(FeaturedStatus::Known {
            guaranteed,
            losing_streak,
            losses_until_radiance,
            next_featured_chance,
            chance_source,
        }) => {
            assert!(!guaranteed, "гарант использован на Мите");
            assert_eq!(losing_streak, 1, "Дилюк — проигранное 50/50");
            assert_eq!(losses_until_radiance, Some(2));
            assert_eq!(next_featured_chance.basis_points(), 5_500);
            assert_eq!(chance_source, ChanceSource::Base);
        }
        other => panic!("ожидался известный статус 5★, получено {other:?}"),
    }
    match character.four_star_featured {
        Some(FeaturedStatus::Known { guaranteed, .. }) => {
            assert!(guaranteed, "4★ не с баннера → гарант 4★")
        }
        other => panic!("ожидался известный статус 4★, получено {other:?}"),
    }
}
