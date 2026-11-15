//! Модульные тесты для `reference.rs`.
//!
//! Подключаются в `reference.rs` строками `#[cfg(test)] #[path = "reference.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Текст с разными вариантами на двух языках.
fn text(en: &str, ru: &str) -> LocalizedText {
    LocalizedText {
        en: en.into(),
        ru: ru.into(),
    }
}

/// Время баннера по местному времени сервера.
fn at(time: &str) -> ScheduleTime {
    ScheduleTime {
        time: WishTime::parse(time).unwrap(),
        asia_based: false,
    }
}

/// Баннер указанного пула с одним предметом 5★.
fn banner(pool: &str, featured: &str) -> BannerInfo {
    BannerInfo {
        pool_id: PoolId::new(pool),
        name: "Test".into(),
        short_name: "Test".into(),
        version: Some("1.0".into()),
        start: at("2026-01-01 00:00:00"),
        end: at("2026-01-21 00:00:00"),
        featured: vec![ItemId::new(featured)],
        featured_rare: vec![],
    }
}

/// Маленький справочник: два предмета и два баннера разных пулов.
fn sample() -> ReferenceSnapshot {
    ReferenceSnapshot {
        source_revision: "abc".into(),
        generated_at: "2026-11-15 00:00:00".into(),
        items: vec![
            ItemInfo {
                id: ItemId::new("hu_tao"),
                kind: ItemKind::Character,
                rarity: Rarity::Five,
                name: text("Hu Tao", "Ху Тао"),
            },
            ItemInfo {
                id: ItemId::new("cool_steel"),
                kind: ItemKind::Weapon,
                rarity: Rarity::Three,
                name: text("Cool Steel", "Холодное лезвие"),
            },
        ],
        banners: vec![
            banner("character", "hu_tao"),
            banner("weapon", "staff_of_homa"),
        ],
    }
}

#[test]
fn localized_text_returns_language_variant() {
    let name = text("Hu Tao", "Ху Тао");
    assert_eq!(name.get(Language::English), "Hu Tao");
    assert_eq!(name.get(Language::Russian), "Ху Тао");
}

#[test]
fn item_is_found_by_id() {
    let data = ReferenceData::new(sample());

    let item = data.item(&ItemId::new("cool_steel")).unwrap();

    assert_eq!(item.name.ru, "Холодное лезвие");
    assert!(data.item(&ItemId::new("missing")).is_none());
}

#[test]
fn banners_are_filtered_by_pool() {
    let data = ReferenceData::new(sample());
    let pool = PoolId::new("character");

    let featured: Vec<&str> = data
        .banners_of(&pool)
        .map(|banner| banner.featured[0].as_str())
        .collect();

    assert_eq!(featured, ["hu_tao"]);
}

#[test]
fn snapshot_roundtrips_through_json() {
    let json = serde_json::to_string(&sample()).unwrap();
    assert_eq!(
        serde_json::from_str::<ReferenceSnapshot>(&json).unwrap(),
        sample()
    );
}
