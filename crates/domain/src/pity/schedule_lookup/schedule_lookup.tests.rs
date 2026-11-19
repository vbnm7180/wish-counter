//! Модульные тесты для `schedule_lookup.rs`.
//!
//! Подключаются в `schedule_lookup.rs` строками
//! `#[cfg(test)] #[path = "schedule_lookup.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Расписание повторяет реальные баннеры 7.0: Odette (начало сразу после
//! обновления — время по UTC+8) и Flins (вторая половина — местное время).

use super::*;
use crate::pity::rules::PityRules;
use crate::reference::ReferenceSnapshot;
use crate::wish::{ItemKind, WishId, WishTime};

/// Код баннера персонажа в тестах.
const CHARACTER_CODE: BannerCode = BannerCode::new(301);
/// Код баннера, которого нет ни в одном пуле.
const UNKNOWN_CODE: BannerCode = BannerCode::new(999);

/// Время расписания.
fn at(time: &str, asia_based: bool) -> ScheduleTime {
    ScheduleTime {
        time: WishTime::parse(time).unwrap(),
        asia_based,
    }
}

/// Баннер пула `character`.
fn banner(start: ScheduleTime, end: ScheduleTime, featured: &[&str], rare: &[&str]) -> BannerInfo {
    let ids = |list: &[&str]| list.iter().map(|id| ItemId::new(*id)).collect();
    BannerInfo {
        pool_id: PoolId::new("character"),
        name: "Test".into(),
        short_name: "Test".into(),
        version: Some("7.0".into()),
        start,
        end,
        featured: ids(featured),
        featured_rare: ids(rare),
    }
}

/// Справочник с двумя баннерами версии 7.0.
fn reference() -> ReferenceData {
    ReferenceData::new(ReferenceSnapshot {
        source_revision: "test".into(),
        generated_at: "2026-11-15 00:00:00".into(),
        items: vec![],
        banners: vec![
            banner(
                at("2026-08-12 06:00:00", true),
                at("2026-09-01 17:59:00", false),
                &["odette", "arlecchino"],
                &["sucrose"],
            ),
            banner(
                at("2026-09-01 18:00:00", false),
                at("2026-09-22 14:59:00", false),
                &["flins", "ineffa"],
                &["aino"],
            ),
        ],
    })
}

/// Пул персонажей с кодом 301.
fn pools() -> Vec<PoolDefinition> {
    vec![PoolDefinition {
        id: PoolId::new("character"),
        banner_codes: vec![CHARACTER_CODE],
        rules: PityRules {
            five_star_hard_pity: 90,
            four_star_hard_pity: 10,
            fifty_fifty: None,
            four_star_featured: None,
        },
        primogems_per_wish: 160,
    }]
}

/// Проверка для сервера `region`.
fn lookup(region: ServerRegion) -> ScheduleFeaturedLookup {
    ScheduleFeaturedLookup::new(&reference(), &pools(), region)
}

/// Молитва на указанном баннере в указанное время.
fn wish(code: BannerCode, time: &str, item: &str, rarity: Rarity) -> Wish {
    Wish {
        id: WishId::new("1"),
        banner_code: code,
        item_id: ItemId::new(item),
        item_kind: ItemKind::Character,
        rarity,
        time: WishTime::parse(time).unwrap(),
    }
}

/// 5★ на баннере персонажа.
fn five(time: &str, item: &str) -> Wish {
    wish(CHARACTER_CODE, time, item, Rarity::Five)
}

#[test]
fn featured_five_star_is_recognized() {
    assert_eq!(
        lookup(ServerRegion::Asia).is_featured(&five("2026-08-20 12:00:00", "odette")),
        Some(true)
    );
}

#[test]
fn off_banner_five_star_is_recognized() {
    assert_eq!(
        lookup(ServerRegion::Asia).is_featured(&five("2026-08-20 12:00:00", "diluc")),
        Some(false)
    );
}

#[test]
fn four_stars_are_checked_against_featured_rare() {
    let lookup = lookup(ServerRegion::Asia);
    let sucrose = wish(
        CHARACTER_CODE,
        "2026-08-20 12:00:00",
        "sucrose",
        Rarity::Four,
    );
    let weapon = wish(
        CHARACTER_CODE,
        "2026-08-20 12:00:00",
        "the_flute",
        Rarity::Four,
    );

    assert_eq!(lookup.is_featured(&sucrose), Some(true));
    assert_eq!(lookup.is_featured(&weapon), Some(false));
}

#[test]
fn three_stars_have_no_featured_status() {
    let wish = wish(
        CHARACTER_CODE,
        "2026-08-20 12:00:00",
        "cool_steel",
        Rarity::Three,
    );
    assert_eq!(lookup(ServerRegion::Asia).is_featured(&wish), None);
}

/// Баннер «сразу после обновления» открывается одновременно по всему миру:
/// 06:00 по UTC+8 — это 17:00 предыдущего дня на американском сервере.
#[test]
fn asia_based_start_is_shifted_for_america() {
    let america = lookup(ServerRegion::America);
    let asia = lookup(ServerRegion::Asia);

    assert_eq!(
        america.is_featured(&five("2026-08-11 17:30:00", "odette")),
        Some(true)
    );
    assert_eq!(
        asia.is_featured(&five("2026-08-11 17:30:00", "odette")),
        None
    );
}

/// Баннер второй половины открывается в 18:00 по местному времени каждого сервера.
#[test]
fn local_start_is_not_shifted() {
    let america = lookup(ServerRegion::America);

    assert_eq!(
        america.is_featured(&five("2026-09-01 18:00:00", "flins")),
        Some(true)
    );
    assert_eq!(
        america.is_featured(&five("2026-09-01 17:59:30", "flins")),
        Some(false)
    );
}

/// Молитва в последнюю минуту баннера (конец записан как 17:59:00) ещё на нём.
#[test]
fn last_minute_of_banner_is_included() {
    assert_eq!(
        lookup(ServerRegion::Europe).is_featured(&five("2026-09-01 17:59:45", "odette")),
        Some(true)
    );
}

#[test]
fn time_outside_any_banner_is_unknown() {
    assert_eq!(
        lookup(ServerRegion::Asia).is_featured(&five("2027-01-01 00:00:00", "odette")),
        None
    );
}

#[test]
fn unknown_banner_code_is_unknown() {
    let wish = wish(UNKNOWN_CODE, "2026-08-20 12:00:00", "odette", Rarity::Five);
    assert_eq!(lookup(ServerRegion::Asia).is_featured(&wish), None);
}

/// Название баннера, на котором сделана молитва.
#[test]
fn banner_name_is_found_for_wish() {
    let lookup = lookup(ServerRegion::Asia);

    assert_eq!(
        lookup.banner_name(&five("2026-08-20 12:00:00", "odette")),
        Some("Test")
    );
    assert_eq!(
        lookup.banner_name(&five("2027-01-01 00:00:00", "odette")),
        None
    );
}
