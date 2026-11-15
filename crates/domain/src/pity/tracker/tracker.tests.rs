//! Модульные тесты для `tracker.rs`.
//!
//! Подключаются в `tracker.rs` строками `#[cfg(test)] #[path = "tracker.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Тесты используют правила баннера персонажа Genshin (90 / 10, 50/50,
//! Capturing Radiance после 3 проигрышей, итоговый шанс 55%).

use super::*;
use crate::pity::rules::CapturingRadianceRule;
use crate::wish::{BannerCode, ItemId, ItemKind, WishId, WishTime};

/// Предмет с баннера (тестовый `FeaturedLookup` отвечает `Some(true)`).
const FEATURED: &str = "featured";
/// Предмет не с баннера (`Some(false)`).
const OFF_BANNER: &str = "off_banner";
/// Предмет, о котором нет данных (`None`).
const UNKNOWN: &str = "unknown";
/// Обычный предмет 3★/4★.
const FILLER: &str = "filler";

/// Тестовая проверка «с баннера ли предмет» по его идентификатору.
struct FakeLookup;

impl FeaturedLookup for FakeLookup {
    fn is_featured(&self, wish: &Wish) -> Option<bool> {
        match wish.item_id.as_str() {
            FEATURED => Some(true),
            OFF_BANNER => Some(false),
            _ => None,
        }
    }
}

/// Правила как у баннера персонажа Genshin.
fn character_rules() -> PityRules {
    PityRules {
        five_star_hard_pity: 90,
        four_star_hard_pity: 10,
        fifty_fifty: Some(FiftyFiftyRule {
            base_win_chance: Probability::from_basis_points(5_000),
            capturing_radiance: Some(CapturingRadianceRule {
                losses_for_guarantee: 3,
                consolidated_win_chance: Probability::from_basis_points(5_500),
            }),
        }),
        four_star_featured: Some(FiftyFiftyRule {
            base_win_chance: Probability::from_basis_points(5_000),
            capturing_radiance: None,
        }),
    }
}

/// Правила без 50/50 (как у стандартного баннера).
fn standard_rules() -> PityRules {
    PityRules {
        fifty_fifty: None,
        four_star_featured: None,
        ..character_rules()
    }
}

/// Создаёт тестовую молитву.
fn wish(rarity: Rarity, item: &str) -> Wish {
    Wish {
        id: WishId::new("1"),
        banner_code: BannerCode::new(301),
        item_id: ItemId::new(item),
        item_kind: ItemKind::Character,
        rarity,
        time: WishTime::parse("2026-01-01 00:00:00").unwrap(),
    }
}

/// `count` молитв с 3★ подряд.
fn threes(count: usize) -> Vec<Wish> {
    vec![wish(Rarity::Three, FILLER); count]
}

/// Склеивает несколько групп молитв в одну историю.
fn history(parts: Vec<Vec<Wish>>) -> Vec<Wish> {
    parts.concat()
}

/// Считает статистику по правилам баннера персонажа.
fn character_statistics(wishes: &[Wish]) -> PoolStatistics {
    calculate_pool_statistics(wishes, &character_rules(), &FakeLookup)
}

/// Достаёт известный статус 50/50 или падает с понятным сообщением.
fn known_status(
    statistics: &PoolStatistics,
) -> (bool, u32, Option<u32>, Probability, ChanceSource) {
    match statistics.featured {
        Some(FeaturedStatus::Known {
            guaranteed,
            losing_streak,
            losses_until_radiance,
            next_featured_chance,
            chance_source,
        }) => (
            guaranteed,
            losing_streak,
            losses_until_radiance,
            next_featured_chance,
            chance_source,
        ),
        other => panic!("ожидался известный статус 50/50, получено {other:?}"),
    }
}

// ---------- Счётчики гаранта ----------

#[test]
fn empty_history_starts_from_zero() {
    let statistics = character_statistics(&[]);

    assert_eq!(statistics.total_wishes, 0);
    assert_eq!(
        statistics.five_star,
        PityCounter {
            current: 0,
            hard_pity: 90,
            remaining: 90
        }
    );
    assert_eq!(
        statistics.four_star,
        PityCounter {
            current: 0,
            hard_pity: 10,
            remaining: 10
        }
    );
}

#[test]
fn counts_wishes_since_last_five_star() {
    let wishes = history(vec![
        threes(10),
        vec![wish(Rarity::Five, FEATURED)],
        threes(34),
    ]);

    let statistics = character_statistics(&wishes);

    assert_eq!(statistics.total_wishes, 45);
    assert_eq!(statistics.five_star.current, 34);
    assert_eq!(statistics.five_star.remaining, 56);
}

#[test]
fn four_star_resets_only_four_star_counter() {
    let wishes = history(vec![
        threes(20),
        vec![wish(Rarity::Four, FILLER)],
        threes(2),
    ]);

    let statistics = character_statistics(&wishes);

    assert_eq!(statistics.four_star.current, 2);
    assert_eq!(statistics.five_star.current, 23);
}

/// 5★ на 10-й молитве не сбрасывает счётчик 4★: гарант переносится на 11-ю.
#[test]
fn five_star_does_not_reset_four_star_counter() {
    let wishes = history(vec![threes(9), vec![wish(Rarity::Five, FEATURED)]]);

    let statistics = character_statistics(&wishes);

    assert_eq!(statistics.four_star.current, 10);
    assert_eq!(statistics.four_star.remaining, 1);
}

#[test]
fn annotations_contain_pity_of_four_and_five_stars() {
    let wishes = history(vec![
        threes(6),
        vec![wish(Rarity::Four, FILLER)],
        threes(70),
        vec![wish(Rarity::Five, FEATURED)],
    ]);

    let annotations = annotate_wishes(&wishes, &character_rules(), &FakeLookup);

    assert_eq!(annotations.len(), 78);
    assert_eq!(annotations[0], WishAnnotation::default());
    assert_eq!(annotations[6].pity, Some(7));
    assert_eq!(annotations[77].pity, Some(78));
    assert_eq!(annotations[77].outcome, Some(FiftyFiftyOutcome::Won));
}

// ---------- 50/50 и Capturing Radiance ----------

#[test]
fn fresh_banner_shows_consolidated_chance() {
    let (guaranteed, streak, until_radiance, chance, source) =
        known_status(&character_statistics(&[]));

    assert!(!guaranteed);
    assert_eq!(streak, 0);
    assert_eq!(until_radiance, Some(3));
    assert_eq!(chance.basis_points(), 5_500);
    assert_eq!(source, ChanceSource::Base);
}

#[test]
fn lost_fifty_fifty_makes_next_five_star_guaranteed() {
    let wishes = vec![wish(Rarity::Five, OFF_BANNER)];

    let (guaranteed, streak, until_radiance, chance, source) =
        known_status(&character_statistics(&wishes));

    assert!(guaranteed);
    assert_eq!(streak, 1);
    assert_eq!(until_radiance, Some(2));
    assert_eq!(chance, Probability::CERTAIN);
    assert_eq!(source, ChanceSource::Guaranteed);
}

/// Гарантированный 5★ снимает гарант, но не сбрасывает серию проигрышей.
#[test]
fn guaranteed_win_keeps_losing_streak() {
    let wishes = vec![wish(Rarity::Five, OFF_BANNER), wish(Rarity::Five, FEATURED)];

    let annotations = annotate_wishes(&wishes, &character_rules(), &FakeLookup);
    let (guaranteed, streak, ..) = known_status(&character_statistics(&wishes));

    assert_eq!(annotations[1].outcome, Some(FiftyFiftyOutcome::Guaranteed));
    assert!(!guaranteed);
    assert_eq!(streak, 1);
}

#[test]
fn won_fifty_fifty_resets_losing_streak() {
    let wishes = vec![
        wish(Rarity::Five, OFF_BANNER),
        wish(Rarity::Five, FEATURED),
        wish(Rarity::Five, FEATURED),
    ];

    let (_, streak, until_radiance, ..) = known_status(&character_statistics(&wishes));

    assert_eq!(streak, 0);
    assert_eq!(until_radiance, Some(3));
}

/// После трёх проигранных 50/50 подряд следующее 50/50 гарантированно выиграно.
#[test]
fn three_lost_fifty_fifties_trigger_capturing_radiance() {
    let lose_then_guarantee = || vec![wish(Rarity::Five, OFF_BANNER), wish(Rarity::Five, FEATURED)];
    let wishes = history(vec![
        lose_then_guarantee(),
        lose_then_guarantee(),
        lose_then_guarantee(),
    ]);

    let (guaranteed, streak, until_radiance, chance, source) =
        known_status(&character_statistics(&wishes));

    assert!(!guaranteed);
    assert_eq!(streak, 3);
    assert_eq!(until_radiance, Some(0));
    assert_eq!(chance, Probability::CERTAIN);
    assert_eq!(source, ChanceSource::CapturingRadiance);
}

#[test]
fn unknown_featured_data_makes_status_unknown() {
    let wishes = vec![wish(Rarity::Five, UNKNOWN)];

    let statistics = character_statistics(&wishes);
    let annotations = annotate_wishes(&wishes, &character_rules(), &FakeLookup);

    assert_eq!(statistics.featured, Some(FeaturedStatus::Unknown));
    assert_eq!(annotations[0].outcome, Some(FiftyFiftyOutcome::Unknown));
}

#[test]
fn pool_without_fifty_fifty_has_no_featured_status() {
    let wishes = vec![wish(Rarity::Five, OFF_BANNER)];

    let statistics = calculate_pool_statistics(&wishes, &standard_rules(), &FakeLookup);
    let annotations = annotate_wishes(&wishes, &standard_rules(), &FakeLookup);

    assert_eq!(statistics.featured, None);
    assert_eq!(annotations[0].outcome, None);
    assert_eq!(annotations[0].pity, Some(1));
}

/// Без Capturing Radiance используется базовый шанс 50%.
#[test]
fn rule_without_radiance_uses_base_chance() {
    let rules = PityRules {
        fifty_fifty: Some(FiftyFiftyRule {
            base_win_chance: Probability::from_basis_points(5_000),
            capturing_radiance: None,
        }),
        ..character_rules()
    };

    let statistics = calculate_pool_statistics(&[], &rules, &FakeLookup);

    match statistics.featured {
        Some(FeaturedStatus::Known {
            next_featured_chance,
            losses_until_radiance,
            ..
        }) => {
            assert_eq!(next_featured_chance.basis_points(), 5_000);
            assert_eq!(losses_until_radiance, None);
        }
        other => panic!("ожидался известный статус, получено {other:?}"),
    }
}

#[test]
fn featured_status_serializes_with_state_tag() {
    let json = serde_json::to_string(&character_statistics(&[]).featured).unwrap();

    assert!(
        json.starts_with(r#"{"state":"known","guaranteed":false"#),
        "{json}"
    );
    assert!(json.contains(r#""chance_source":"base""#));
    assert!(json.contains(r#""next_featured_chance":5500"#));
}

// ---------- 4★ с баннера ----------

/// Достаёт известный статус «4★ с баннера».
fn four_star_status(statistics: &PoolStatistics) -> (bool, Probability, ChanceSource) {
    match statistics.four_star_featured {
        Some(FeaturedStatus::Known {
            guaranteed,
            next_featured_chance,
            chance_source,
            ..
        }) => (guaranteed, next_featured_chance, chance_source),
        other => panic!("ожидался известный статус 4★, получено {other:?}"),
    }
}

#[test]
fn four_star_starts_with_base_chance() {
    let (guaranteed, chance, source) = four_star_status(&character_statistics(&[]));

    assert!(!guaranteed);
    assert_eq!(chance.basis_points(), 5_000);
    assert_eq!(source, ChanceSource::Base);
}

/// Проигрыш 4★ даёт гарант на следующий 4★ и не влияет на 50/50 для 5★.
#[test]
fn lost_four_star_makes_next_four_star_guaranteed() {
    let wishes = vec![wish(Rarity::Four, OFF_BANNER)];

    let statistics = character_statistics(&wishes);
    let (guaranteed, chance, source) = four_star_status(&statistics);

    assert!(guaranteed);
    assert_eq!(chance, Probability::CERTAIN);
    assert_eq!(source, ChanceSource::Guaranteed);
    let (five_star_guaranteed, ..) = known_status(&statistics);
    assert!(!five_star_guaranteed);
}

#[test]
fn four_star_outcomes_are_annotated() {
    let wishes = vec![wish(Rarity::Four, OFF_BANNER), wish(Rarity::Four, FEATURED)];

    let annotations = annotate_wishes(&wishes, &character_rules(), &FakeLookup);

    assert_eq!(annotations[0].outcome, Some(FiftyFiftyOutcome::Lost));
    assert_eq!(annotations[1].outcome, Some(FiftyFiftyOutcome::Guaranteed));
}

#[test]
fn pool_without_four_star_rule_has_no_four_star_status() {
    let statistics = calculate_pool_statistics(
        &[wish(Rarity::Four, OFF_BANNER)],
        &standard_rules(),
        &FakeLookup,
    );
    assert_eq!(statistics.four_star_featured, None);
}
