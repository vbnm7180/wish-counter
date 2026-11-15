//! Модульные тесты для `in_memory_wish_repository.rs`.
//!
//! Подключаются в `in_memory_wish_repository.rs` строками
//! `#[cfg(test)] #[path = "in_memory_wish_repository.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use domain::wish::{BannerCode, ItemId, ItemKind, Rarity, WishId, WishTime};

use super::*;

/// Создаёт тестовую молитву с указанными идентификатором и временем.
fn wish(id: &str, time: &str) -> Wish {
    Wish {
        id: WishId::new(id),
        banner_code: BannerCode::new(301),
        item_id: ItemId::new("sword"),
        item_kind: ItemKind::Weapon,
        rarity: Rarity::Three,
        time: WishTime::parse(time).unwrap(),
    }
}

/// Возвращает идентификаторы молитв игры по порядку.
fn ids(repository: &InMemoryWishRepository, game: &str) -> Vec<String> {
    repository
        .list_wishes(&GameId::new(game))
        .unwrap()
        .iter()
        .map(|wish| wish.id.as_str().to_owned())
        .collect()
}

#[test]
fn returns_empty_list_for_unknown_game() {
    assert!(ids(&InMemoryWishRepository::new(), "genshin").is_empty());
}

/// Молитвы сортируются по времени, а при равном времени сохраняют порядок.
#[test]
fn returns_wishes_in_chronological_order() {
    let repository = InMemoryWishRepository::new().with_wishes(
        GameId::new("genshin"),
        vec![
            wish("late", "2026-02-01 00:00:00"),
            wish("early-a", "2026-01-01 00:00:00"),
            wish("early-b", "2026-01-01 00:00:00"),
        ],
    );

    assert_eq!(ids(&repository, "genshin"), ["early-a", "early-b", "late"]);
}

#[test]
fn keeps_games_separate() {
    let repository = InMemoryWishRepository::new()
        .with_wishes(
            GameId::new("genshin"),
            vec![wish("g", "2026-01-01 00:00:00")],
        )
        .with_wishes(GameId::new("hsr"), vec![wish("h", "2026-01-01 00:00:00")]);

    assert_eq!(ids(&repository, "genshin"), ["g"]);
}
