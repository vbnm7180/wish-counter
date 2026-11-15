//! Модульные тесты для `wish.rs`.
//!
//! Подключаются в `wish.rs` строками `#[cfg(test)] #[path = "wish.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn rarity_converts_from_stars() {
    assert_eq!(Rarity::try_from(5), Ok(Rarity::Five));
    assert_eq!(Rarity::try_from(2), Err(RarityError::Unsupported(2)));
}

#[test]
fn rarity_serializes_as_number() {
    assert_eq!(serde_json::to_string(&Rarity::Four).unwrap(), "4");
    assert_eq!(serde_json::from_str::<Rarity>("3").unwrap(), Rarity::Three);
    assert!(serde_json::from_str::<Rarity>("6").is_err());
}

#[test]
fn rarity_order_follows_stars() {
    assert!(Rarity::Five > Rarity::Four);
    assert!(Rarity::Four > Rarity::Three);
}

#[test]
fn wish_time_accepts_game_format() {
    assert!(WishTime::parse("2026-11-15 18:04:59").is_ok());
}

#[test]
fn wish_time_rejects_other_formats() {
    for invalid in [
        "2026-11-15",
        "2026-11-15T18:04:59",
        "2026-1x-15 18:04:59",
        "2026-13-45 18:04:59",
        "2026-02-30 00:00:00",
        "",
    ] {
        assert!(
            WishTime::parse(invalid).is_err(),
            "{invalid} должна быть отклонена"
        );
    }
}

/// Строки времени сравниваются в хронологическом порядке.
#[test]
fn wish_times_compare_chronologically() {
    let earlier = WishTime::parse("2025-12-31 23:59:59").unwrap();
    let later = WishTime::parse("2026-01-01 00:00:00").unwrap();
    assert!(earlier < later);
}

/// Идентификатор молитвы сериализуется строкой, чтобы не терять точность в JS.
#[test]
fn wish_id_serializes_as_string() {
    let id = WishId::new("1700000000000000001");
    assert_eq!(
        serde_json::to_string(&id).unwrap(),
        "\"1700000000000000001\""
    );
}

#[test]
fn wish_roundtrips_through_json() {
    let wish = Wish {
        id: WishId::new("1"),
        banner_code: BannerCode::new(301),
        item_id: ItemId::new("hu_tao"),
        item_kind: ItemKind::Character,
        rarity: Rarity::Five,
        time: WishTime::parse("2026-11-15 18:04:59").unwrap(),
    };

    let json = serde_json::to_string(&wish).unwrap();

    assert_eq!(serde_json::from_str::<Wish>(&json).unwrap(), wish);
    assert!(json.contains(r#""item_kind":"character""#));
    assert!(json.contains(r#""banner_code":301"#));
}

/// Перевод в дату и обратно не меняет значение.
#[test]
fn wish_time_roundtrips_through_date_time() {
    let time = WishTime::parse("2026-11-15 18:04:59").unwrap();

    let restored = WishTime::from_date_time(time.to_date_time());

    assert_eq!(restored, time);
}
