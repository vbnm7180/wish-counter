//! Баннеры Genshin Impact и правила их гаранта.
//!
//! Правила сверены с версией 7.x (ноябрь 2026, после изменений 5.0).
//! Источники — в docs/genshin-rules.md.

use domain::pity::rules::{
    CapturingRadianceRule, FiftyFiftyRule, PityRules, PoolDefinition, PoolId, Probability,
};
use domain::wish::BannerCode;

/// Коды баннеров Genshin Impact (`gacha_type` в API истории молитв).
pub mod banner_codes {
    use domain::wish::BannerCode;

    /// Баннер новичка («Новичковая молитва»), не больше 20 молитв.
    pub const BEGINNER: BannerCode = BannerCode::new(100);
    /// Стандартный баннер («Жажда странствий»).
    pub const STANDARD: BannerCode = BannerCode::new(200);
    /// Баннер события персонажа.
    pub const CHARACTER: BannerCode = BannerCode::new(301);
    /// Баннер события оружия.
    pub const WEAPON: BannerCode = BannerCode::new(302);
    /// Второй баннер события персонажа (общий гарант с [`CHARACTER`]).
    pub const CHARACTER_2: BannerCode = BannerCode::new(400);
    /// Хроники (Chronicled Wish), с версии 4.5.
    pub const CHRONICLED: BannerCode = BannerCode::new(500);
}

/// Идентификаторы пулов. Совпадают с ключами перевода `genshin.banner.<id>` в интерфейсе.
pub mod pool_ids {
    /// Баннеры события персонажа (301 + 400).
    pub const CHARACTER: &str = "character";
    /// Баннер события оружия.
    pub const WEAPON: &str = "weapon";
    /// Хроники.
    pub const CHRONICLED: &str = "chronicled";
    /// Стандартный баннер.
    pub const STANDARD: &str = "standard";
    /// Баннер новичка.
    pub const BEGINNER: &str = "beginner";
}

/// Стоимость одной молитвы: 1 Переплетающаяся / Судьбоносная встреча = 160 примогемов.
///
/// Скидка новичкового баннера (10 молитв за 8 встреч) не учитывается:
/// по истории нельзя понять, делались молитвы по одной или по 10 (как и в Paimon.moe).
const PRIMOGEMS_PER_WISH: u32 = 160;
/// Жёсткий гарант 5★ на баннерах персонажей, хрониках, стандартном.
const CHARACTER_FIVE_STAR_HARD_PITY: u32 = 90;
/// Жёсткий гарант 5★ на оружейном баннере.
const WEAPON_FIVE_STAR_HARD_PITY: u32 = 80;
/// Гарант 4★ или выше — каждые 10 молитв на всех баннерах.
const FOUR_STAR_HARD_PITY: u32 = 10;
/// Базовый шанс выиграть 50/50 на баннере персонажа.
const CHARACTER_BASE_WIN_CHANCE: Probability = Probability::from_basis_points(5_000);
/// Итоговый шанс выиграть 50/50 с Capturing Radiance (официально 55,000%).
const CHARACTER_CONSOLIDATED_WIN_CHANCE: Probability = Probability::from_basis_points(5_500);
/// После скольких проигранных 50/50 подряд Capturing Radiance гарантирован.
const LOSSES_FOR_CAPTURING_RADIANCE: u32 = 3;

/// Возвращает все пулы баннеров Genshin Impact в порядке отображения.
pub fn genshin_pools() -> Vec<PoolDefinition> {
    vec![
        character_pool(),
        weapon_pool(),
        chronicled_pool(),
        standard_pool(),
        beginner_pool(),
    ]
}

/// Баннеры события персонажа: 90 / 10, 50/50 с Capturing Radiance.
fn character_pool() -> PoolDefinition {
    pool(
        pool_ids::CHARACTER,
        vec![banner_codes::CHARACTER, banner_codes::CHARACTER_2],
        PityRules {
            five_star_hard_pity: CHARACTER_FIVE_STAR_HARD_PITY,
            four_star_hard_pity: FOUR_STAR_HARD_PITY,
            fifty_fifty: Some(FiftyFiftyRule {
                base_win_chance: CHARACTER_BASE_WIN_CHANCE,
                capturing_radiance: Some(CapturingRadianceRule {
                    losses_for_guarantee: LOSSES_FOR_CAPTURING_RADIANCE,
                    consolidated_win_chance: CHARACTER_CONSOLIDATED_WIN_CHANCE,
                }),
            }),
        },
    )
}

/// Баннер оружия: 80 / 10.
///
/// Гарант выбранного оружия (Epitomized Path) по истории не вычислить:
/// в ней не записано, какое оружие выбрано целью. Поэтому без правила 50/50.
fn weapon_pool() -> PoolDefinition {
    pool(
        pool_ids::WEAPON,
        vec![banner_codes::WEAPON],
        rules_without_fifty_fifty(WEAPON_FIVE_STAR_HARD_PITY),
    )
}

/// Хроники: 90 / 10. Гарант (Chronicled Path) зависит от выбранной цели,
/// которой нет в истории, — как у оружейного баннера.
fn chronicled_pool() -> PoolDefinition {
    pool(
        pool_ids::CHRONICLED,
        vec![banner_codes::CHRONICLED],
        rules_without_fifty_fifty(CHARACTER_FIVE_STAR_HARD_PITY),
    )
}

/// Стандартный баннер: 90 / 10, все 5★ «с баннера».
fn standard_pool() -> PoolDefinition {
    pool(
        pool_ids::STANDARD,
        vec![banner_codes::STANDARD],
        rules_without_fifty_fifty(CHARACTER_FIVE_STAR_HARD_PITY),
    )
}

/// Баннер новичка: считается отдельно от стандартного (как в Paimon.moe).
fn beginner_pool() -> PoolDefinition {
    pool(
        pool_ids::BEGINNER,
        vec![banner_codes::BEGINNER],
        rules_without_fifty_fifty(CHARACTER_FIVE_STAR_HARD_PITY),
    )
}

/// Правила без 50/50 с заданным гарантом 5★.
fn rules_without_fifty_fifty(five_star_hard_pity: u32) -> PityRules {
    PityRules {
        five_star_hard_pity,
        four_star_hard_pity: FOUR_STAR_HARD_PITY,
        fifty_fifty: None,
    }
}

/// Собирает описание пула со стандартной стоимостью молитвы.
fn pool(id: &str, banner_codes: Vec<BannerCode>, rules: PityRules) -> PoolDefinition {
    PoolDefinition {
        id: PoolId::new(id),
        banner_codes,
        rules,
        primogems_per_wish: PRIMOGEMS_PER_WISH,
    }
}

// Тесты лежат рядом, в файле `pools.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "pools.tests.rs"]
mod tests;
