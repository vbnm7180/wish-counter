//! Модульные тесты для `game_catalog.rs`.
//!
//! Подключаются в `game_catalog.rs` строками `#[cfg(test)] #[path = "game_catalog.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use domain::pity::featured::{FeaturedLookup, NoFeaturedData};
use domain::pity::rules::PoolDefinition;

use super::*;

/// Тестовый игровой модуль с заданными идентификатором и порядком.
struct FakeGame {
    id: &'static str,
    order: u32,
}

impl GameModule for FakeGame {
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new(self.id),
            display_order: self.order,
        }
    }

    fn pools(&self) -> Vec<PoolDefinition> {
        Vec::new()
    }

    fn featured_lookup(&self) -> Arc<dyn FeaturedLookup> {
        Arc::new(NoFeaturedData)
    }
}

/// Собирает каталог из двух тестовых игр в «неправильном» порядке.
fn catalog_with_two_games() -> GameCatalog {
    GameCatalog::new()
        .with_module(Arc::new(FakeGame {
            id: "second",
            order: 20,
        }))
        .with_module(Arc::new(FakeGame {
            id: "first",
            order: 10,
        }))
}

#[test]
fn list_games_is_sorted_by_display_order() {
    let ids: Vec<String> = catalog_with_two_games()
        .list_games()
        .into_iter()
        .map(|game| game.id.to_string())
        .collect();

    assert_eq!(ids, ["first", "second"]);
}

#[test]
fn find_returns_registered_module() {
    let catalog = catalog_with_two_games();
    assert!(catalog.find(&GameId::new("first")).is_some());
}

#[test]
fn find_returns_none_for_unknown_game() {
    let catalog = catalog_with_two_games();
    assert!(catalog.find(&GameId::new("unknown")).is_none());
}
