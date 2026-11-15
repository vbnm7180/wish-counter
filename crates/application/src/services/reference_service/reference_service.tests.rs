//! Модульные тесты для `reference_service.rs`.
//!
//! Подключаются в `reference_service.rs` строками
//! `#[cfg(test)] #[path = "reference_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use domain::game::{GameInfo, GameModule};
use domain::pity::featured::{FeaturedLookup, NoFeaturedData};
use domain::pity::rules::{PoolDefinition, PoolId};
use domain::reference::{ItemInfo, LocalizedText, ReferenceSnapshot, ScheduleTime};
use domain::wish::{ItemId, Rarity, WishTime};

use super::*;

/// Тестовая игра с маленьким справочником: 2 персонажа, 1 оружие, 2 баннера.
struct FakeGame;

/// Создаёт предмет справочника.
fn item(id: &str, kind: ItemKind) -> ItemInfo {
    ItemInfo {
        id: ItemId::new(id),
        kind,
        rarity: Rarity::Five,
        name: LocalizedText {
            en: id.into(),
            ru: id.into(),
        },
    }
}

/// Создаёт баннер с указанным началом.
fn banner(short_name: &str, start: &str) -> BannerInfo {
    let at = |time: &str| ScheduleTime {
        time: WishTime::parse(time).unwrap(),
        asia_based: false,
    };
    BannerInfo {
        pool_id: PoolId::new("character"),
        name: short_name.into(),
        short_name: short_name.into(),
        version: Some("7.2".into()),
        start: at(start),
        end: at("2030-01-01 00:00:00"),
        featured: vec![],
        featured_rare: vec![],
    }
}

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

    fn featured_lookup(&self) -> Arc<dyn FeaturedLookup> {
        Arc::new(NoFeaturedData)
    }

    fn embedded_reference(&self) -> ReferenceSnapshot {
        ReferenceSnapshot {
            source_revision: "abc".into(),
            generated_at: "2026-11-15 00:00:00".into(),
            items: vec![
                item("hu_tao", ItemKind::Character),
                item("mitya", ItemKind::Character),
                item("staff_of_homa", ItemKind::Weapon),
            ],
            banners: vec![
                banner("Mitya", "2026-11-04 06:00:00"),
                banner("Hu Tao", "2021-03-02 18:00:00"),
            ],
        }
    }
}

/// Сервис с одной тестовой игрой.
fn service() -> ReferenceService {
    ReferenceService::from_catalog(&GameCatalog::new().with_module(Arc::new(FakeGame)))
}

#[test]
fn summary_counts_items_and_banners() {
    let summary = service().summary(&GameId::new("fake")).unwrap();

    assert_eq!(summary.character_count, 2);
    assert_eq!(summary.weapon_count, 1);
    assert_eq!(summary.banner_count, 2);
    assert_eq!(summary.source_revision, "abc");
}

/// Последним считается баннер с самым поздним началом, а не последний в списке.
#[test]
fn summary_finds_latest_banner_by_start_time() {
    let latest = service()
        .summary(&GameId::new("fake"))
        .unwrap()
        .latest_banner
        .unwrap();

    assert_eq!(latest.short_name, "Mitya");
    assert_eq!(latest.start, "2026-11-04 06:00:00");
}

#[test]
fn unknown_game_is_reported() {
    assert!(matches!(
        service().summary(&GameId::new("missing")),
        Err(ReferenceError::UnknownGame(_))
    ));
}
