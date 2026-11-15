//! Сквозной тест: SQLite → StatisticsService → правила Genshin.
//!
//! В отличие от модульных тестов (рядом с кодом), этот файл лежит в папке
//! `tests/` крейта: Cargo собирает его как отдельную программу, которая
//! видит только публичный API. Здесь проверяется, что слои правильно
//! стыкуются друг с другом — так же, как в `bootstrap.rs`.

use std::sync::Arc;

use application::ports::WishRepository;
use application::services::{GameCatalog, PoolSummary, StatisticsService};
use domain::game::GameId;
use domain::pity::statistics::FeaturedStatus;
use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, Wish, WishId, WishTime};
use game_genshin::{GENSHIN_GAME_ID, GenshinModule, banner_codes, pool_ids};
use infrastructure::database::open_in_memory_database;
use infrastructure::wishes::SqliteWishRepository;

/// Создаёт молитву с номером `index` (номер задаёт и id, и время по порядку).
fn wish(index: u32, banner: BannerCode, rarity: Rarity) -> Wish {
    Wish {
        id: WishId::new(format!("{index:019}")),
        banner_code: banner,
        item_id: ItemId::new(if rarity == Rarity::Five {
            "diluc"
        } else {
            "cool_steel"
        }),
        item_kind: ItemKind::Weapon,
        rarity,
        time: WishTime::parse(format!("2026-01-01 00:{:02}:{:02}", index / 60, index % 60))
            .unwrap(),
    }
}

/// Находит карточку пула по идентификатору.
fn pool<'a>(pools: &'a [PoolSummary], id: &str) -> &'a PoolSummary {
    pools
        .iter()
        .find(|pool| pool.pool_id.as_str() == id)
        .unwrap()
}

#[test]
fn statistics_are_calculated_from_sqlite_history() {
    let genshin = GameId::new(GENSHIN_GAME_ID);
    let repository = Arc::new(SqliteWishRepository::new(
        open_in_memory_database().unwrap(),
    ));
    // 85 молитв на двух баннерах персонажа (общий счётчик), 5★ на 75-й; 3 молитвы на оружейном.
    let history: Vec<Wish> = (1..=85)
        .map(|index| {
            let banner = if index % 2 == 0 {
                banner_codes::CHARACTER
            } else {
                banner_codes::CHARACTER_2
            };
            let rarity = if index == 75 {
                Rarity::Five
            } else {
                Rarity::Three
            };
            wish(index, banner, rarity)
        })
        .chain((86..=88).map(|index| wish(index, banner_codes::WEAPON, Rarity::Three)))
        .collect();
    repository.add_wishes(&genshin, &history).unwrap();

    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(GenshinModule)));
    let statistics = StatisticsService::new(catalog, repository)
        .game_statistics(&genshin)
        .unwrap();

    let character = pool(&statistics.pools, pool_ids::CHARACTER);
    assert_eq!(character.statistics.total_wishes, 85);
    assert_eq!(character.statistics.five_star.current, 10);
    assert_eq!(character.statistics.five_star.remaining, 80);
    assert_eq!(character.primogems_spent, 85 * 160);
    // Данных о баннерах ещё нет (этап 4), поэтому исход 50/50 неизвестен.
    assert_eq!(character.statistics.featured, Some(FeaturedStatus::Unknown));

    let weapon = pool(&statistics.pools, pool_ids::WEAPON);
    assert_eq!(weapon.statistics.total_wishes, 3);
    assert_eq!(weapon.statistics.five_star.remaining, 77);

    assert_eq!(statistics.total_wishes, 88);
    assert_eq!(statistics.primogems_spent, 88 * 160);
}
