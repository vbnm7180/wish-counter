//! Модульные тесты для `pools.rs`.
//!
//! Подключаются в `pools.rs` строками `#[cfg(test)] #[path = "pools.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::collections::HashSet;

use super::*;

/// Находит пул по идентификатору или падает.
fn find_pool(id: &str) -> PoolDefinition {
    genshin_pools()
        .into_iter()
        .find(|pool| pool.id.as_str() == id)
        .unwrap_or_else(|| panic!("пул {id} не найден"))
}

#[test]
fn pool_ids_are_unique() {
    let pools = genshin_pools();
    let ids: HashSet<&str> = pools.iter().map(|pool| pool.id.as_str()).collect();
    assert_eq!(ids.len(), pools.len());
}

/// Каждый код баннера относится ровно к одному пулу.
#[test]
fn every_banner_code_belongs_to_exactly_one_pool() {
    let codes = [
        banner_codes::BEGINNER,
        banner_codes::STANDARD,
        banner_codes::CHARACTER,
        banner_codes::WEAPON,
        banner_codes::CHARACTER_2,
        banner_codes::CHRONICLED,
    ];
    for code in codes {
        let owners = genshin_pools()
            .iter()
            .filter(|pool| pool.contains(code))
            .count();
        assert_eq!(
            owners,
            1,
            "код {} должен быть ровно в одном пуле",
            code.value()
        );
    }
}

/// Оба баннера персонажа делят общий счётчик гаранта.
#[test]
fn character_banners_share_one_pool() {
    let pool = find_pool(pool_ids::CHARACTER);
    assert!(pool.contains(banner_codes::CHARACTER));
    assert!(pool.contains(banner_codes::CHARACTER_2));
}

#[test]
fn character_pool_has_capturing_radiance() {
    let rules = find_pool(pool_ids::CHARACTER).rules;
    let radiance = rules
        .fifty_fifty
        .and_then(|rule| rule.capturing_radiance)
        .unwrap();

    assert_eq!(rules.five_star_hard_pity, 90);
    assert_eq!(radiance.losses_for_guarantee, 3);
    assert_eq!(radiance.consolidated_win_chance.basis_points(), 5_500);
}

#[test]
fn weapon_pool_has_hard_pity_80() {
    assert_eq!(find_pool(pool_ids::WEAPON).rules.five_star_hard_pity, 80);
}

#[test]
fn every_pool_has_four_star_pity_10_and_costs_160() {
    for pool in genshin_pools() {
        assert_eq!(pool.rules.four_star_hard_pity, 10);
        assert_eq!(pool.primogems_for(10), 1_600);
    }
}
