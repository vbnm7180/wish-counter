//! Модульные тесты для `backup_export.rs`.
//!
//! Подключаются в `backup_export.rs` строками
//! `#[cfg(test)] #[path = "backup_export.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Используются настоящий справочник и правила Genshin: молитвы — на баннере Мити (7.2).

use domain::import::HistoryParser;
use domain::profile::{GameProfile, PlayerUid};
use domain::reference::ReferenceData;
use domain::wish::{BannerCode, ItemId, WishId, WishTime};

use super::*;
use crate::banner_codes;
use crate::paimon::backup::PaimonBackupParser;
use crate::pools::genshin_pools;
use crate::reference::embedded_reference;

/// Молитва номер `index` (каждая следующая на минуту позже, середина баннера Мити).
fn wish(index: u32, code: BannerCode, item: &str, kind: ItemKind, rarity: Rarity) -> Wish {
    Wish {
        id: WishId::new(format!("{index}")),
        banner_code: code,
        item_id: ItemId::new(item),
        item_kind: kind,
        rarity,
        time: WishTime::parse(format!("2026-11-10 12:{index:02}:00")).unwrap(),
    }
}

/// История: на баннере персонажа 3★, 4★ не с баннера, Дилюк (проигрыш), Митя (гарант);
/// на стандартном — одна 3★.
fn history() -> Vec<Wish> {
    vec![
        wish(
            1,
            banner_codes::CHARACTER,
            "cool_steel",
            ItemKind::Weapon,
            Rarity::Three,
        ),
        wish(
            2,
            banner_codes::CHARACTER,
            "the_flute",
            ItemKind::Weapon,
            Rarity::Four,
        ),
        wish(
            3,
            banner_codes::CHARACTER,
            "diluc",
            ItemKind::Character,
            Rarity::Five,
        ),
        wish(
            4,
            banner_codes::CHARACTER_2,
            "mitya",
            ItemKind::Character,
            Rarity::Five,
        ),
        wish(
            5,
            banner_codes::STANDARD,
            "cool_steel",
            ItemKind::Weapon,
            Rarity::Three,
        ),
    ]
}

/// Экспортирует историю с указанным профилем и возвращает разобранный JSON.
fn export(profile: &GameProfile) -> Value {
    let wishes = history();
    let pools = genshin_pools();
    let reference = ReferenceData::new(embedded_reference());
    let input = ExportInput {
        wishes: &wishes,
        pools: &pools,
        reference: &reference,
        profile,
    };
    serde_json::from_str(&PaimonBackupExporter.export(&input).unwrap()).unwrap()
}

/// Профиль: Европа и UID.
fn europe() -> GameProfile {
    GameProfile {
        server_region: Some(ServerRegion::Europe),
        uid: Some(PlayerUid::parse("700000000").unwrap()),
    }
}

#[test]
fn writes_server_uid_and_non_empty_sections_only() {
    let json = export(&europe());

    assert_eq!(json["server"], "Europe");
    assert_eq!(json["wish-uid"], "700000000");
    assert!(json.get("wish-counter-character-event").is_some());
    assert!(json.get("wish-counter-standard").is_some());
    assert!(json.get("wish-counter-weapon-event").is_none());
}

/// Счётчики, `pity` и `rate` — как считает Paimon.moe.
#[test]
fn character_section_matches_paimon_rules() {
    let json = export(&europe());
    let section = &json["wish-counter-character-event"];
    let pulls = section["pulls"].as_array().unwrap();

    assert_eq!(section["total"], 4);
    assert_eq!(section["legendary"], 0);
    assert_eq!(section["rare"], 2);
    // 3★: pity 1, без rate.
    assert_eq!(pulls[0]["pity"], 1);
    assert!(pulls[0].get("rate").is_none());
    // 4★ не с баннера: rate 0; Дилюк — проигрыш (0); Митя — по гаранту (2).
    assert_eq!(pulls[1]["rate"], 0);
    assert_eq!(pulls[2]["pity"], 3);
    assert_eq!(pulls[2]["rate"], 0);
    assert_eq!(pulls[3]["pity"], 1);
    assert_eq!(pulls[3]["rate"], 2);
    assert_eq!(pulls[3]["code"], "400");
    assert_eq!(pulls[3]["type"], "character");
    // Гарант 5★ использован; 4★ проиграно — следующий 4★ гарантирован.
    assert_eq!(section["guaranteed"]["legendary"], false);
    assert_eq!(section["guaranteed"]["rare"], true);
    assert_eq!(section["rateoff"]["legendary"]["lose"], 1);
    assert_eq!(section["rateoff"]["legendary"]["win"], 0);
}

/// Без сервера отметки 50/50 не пишутся (так же поступает Paimon.moe).
#[test]
fn without_region_rates_are_omitted() {
    let json = export(&GameProfile::default());
    let section = &json["wish-counter-character-event"];

    assert!(json.get("server").is_none());
    assert!(section.get("guaranteed").is_none());
    assert!(section["pulls"][2].get("rate").is_none());
    assert_eq!(section["pulls"][2]["pity"], 3);
}

/// Экспорт и обратный импорт дают те же молитвы, сервер и UID.
#[test]
fn export_then_import_roundtrip() {
    let reference = ReferenceData::new(embedded_reference());
    let json = export(&europe()).to_string();

    let parsed = PaimonBackupParser.parse(&json, &reference).unwrap();

    let key = |wish: &Wish| {
        (
            wish.banner_code,
            wish.item_id.clone(),
            wish.time.clone(),
            wish.rarity,
        )
    };
    let original: Vec<_> = history().iter().map(key).collect();
    let restored: Vec<_> = parsed.wishes.iter().map(key).collect();
    assert_eq!(restored, original);
    assert_eq!(parsed.server_region, Some(ServerRegion::Europe));
    assert_eq!(parsed.uid, europe().uid);
}

#[test]
fn rate_off_tracks_streaks_like_paimon() {
    let mut rates = RateOff::default();
    for outcome in [
        FiftyFiftyOutcome::Won,
        FiftyFiftyOutcome::Won,
        FiftyFiftyOutcome::Lost,
        FiftyFiftyOutcome::Guaranteed,
        FiftyFiftyOutcome::Won,
    ] {
        rates.record(outcome);
    }

    let rates = rates.finish();

    assert_eq!((rates.win, rates.lose), (3, 1));
    assert_eq!((rates.max_streak, rates.current_streak), (2, 1));
}
