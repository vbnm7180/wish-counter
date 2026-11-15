//! Правила гаранта и пулы баннеров.

use serde::{Deserialize, Serialize};

use crate::wish::BannerCode;

/// Идентификатор пула баннеров, например `"character"`.
///
/// **Пул** — группа баннеров с общим счётчиком гаранта. Например, в Genshin
/// оба баннера события персонажа (коды 301 и 400) входят в один пул.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PoolId(String);

impl PoolId {
    /// Создаёт идентификатор пула.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Возвращает идентификатор как строковый срез.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Количество сотых долей процента в 100%.
const BASIS_POINTS_IN_WHOLE: u32 = 10_000;

/// Вероятность, хранимая в сотых долях процента (basis points):
/// `5500` = 55,00%, `10000` = 100%.
///
/// Целое число вместо `f64` выбрано, чтобы значения сравнивались точно
/// (у дробных чисел бывают ошибки округления: `0.1 + 0.2 != 0.3`).
/// В JSON передаётся числом, интерфейс делит его на 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Probability(u32);

impl Probability {
    /// Вероятность 100%: событие гарантировано.
    pub const CERTAIN: Self = Self(BASIS_POINTS_IN_WHOLE);

    /// Создаёт вероятность из сотых долей процента.
    ///
    /// Значение больше 100% — ошибка программиста, поэтому функция
    /// вызывает `panic!`. В константах такая ошибка обнаружится
    /// ещё при компиляции.
    pub const fn from_basis_points(basis_points: u32) -> Self {
        assert!(
            basis_points <= BASIS_POINTS_IN_WHOLE,
            "вероятность не может превышать 100%"
        );
        Self(basis_points)
    }

    /// Возвращает вероятность в сотых долях процента.
    pub const fn basis_points(self) -> u32 {
        self.0
    }
}

/// Правило Capturing Radiance (Genshin, с версии 5.0).
///
/// Скрытая механика повышает шанс выиграть 50/50. Официально известны
/// только итоговый шанс и правило «после N проигрышей подряд — гарантия».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturingRadianceRule {
    /// Сколько 50/50 нужно проиграть подряд, чтобы следующее было гарантированно выиграно.
    pub losses_for_guarantee: u32,
    /// Итоговый шанс выиграть 50/50 с учётом механики (у Genshin — 55%).
    pub consolidated_win_chance: Probability,
}

/// Правило «50/50»: выпавший 5★ может оказаться не предметом с баннера.
/// После проигрыша следующий 5★ гарантированно будет с баннера.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiftyFiftyRule {
    /// Базовый шанс выиграть (обычно 50%).
    pub base_win_chance: Probability,
    /// Capturing Radiance, если игра его поддерживает.
    pub capturing_radiance: Option<CapturingRadianceRule>,
}

/// Правила гаранта одного пула баннеров.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PityRules {
    /// Не позже какой молитвы гарантирован 5★ (жёсткий гарант).
    pub five_star_hard_pity: u32,
    /// Не позже какой молитвы гарантирован 4★ или выше.
    pub four_star_hard_pity: u32,
    /// Правило 50/50; `None`, если на баннере его нет (стандартный)
    /// или его нельзя определить по истории (оружейный — см. docs/roadmap.md).
    pub fifty_fifty: Option<FiftyFiftyRule>,
}

/// Описание пула баннеров: какие баннеры входят, правила и стоимость.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolDefinition {
    /// Идентификатор пула.
    pub id: PoolId,
    /// Коды баннеров, которые делят общий счётчик гаранта.
    pub banner_codes: Vec<BannerCode>,
    /// Правила гаранта.
    pub rules: PityRules,
    /// Стоимость одной молитвы в премиальной валюте (в Genshin — 160 примогемов).
    pub primogems_per_wish: u32,
}

impl PoolDefinition {
    /// Проверяет, относится ли баннер с этим кодом к пулу.
    pub fn contains(&self, code: BannerCode) -> bool {
        self.banner_codes.contains(&code)
    }

    /// Сколько примогемов стоят указанное число молитв.
    ///
    /// Результат `u64`: при `u32` переполнение было бы теоретически возможно.
    pub fn primogems_for(&self, wish_count: u32) -> u64 {
        u64::from(wish_count) * u64::from(self.primogems_per_wish)
    }
}
