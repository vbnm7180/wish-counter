//! Результаты расчёта гаранта: то, что показывается пользователю.

use serde::Serialize;

use super::rules::Probability;

/// Счётчик гаранта одной редкости (5★ или 4★).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PityCounter {
    /// Сколько молитв сделано после последнего предмета этой редкости.
    pub current: u32,
    /// Жёсткий гарант: не позже какой молитвы предмет гарантирован.
    pub hard_pity: u32,
    /// Сколько молитв осталось сделать, чтобы предмет был гарантирован
    /// (включая гарантированную молитву). Всегда не меньше 1.
    pub remaining: u32,
}

impl PityCounter {
    /// Создаёт счётчик и вычисляет, сколько молитв осталось до гаранта.
    ///
    /// Счётчик 4★ может превысить жёсткий гарант: если на 10-й молитве
    /// выпал 5★, гарантированный 4★ переносится на 11-ю. Поэтому остаток
    /// ограничен снизу единицей — «гарант на следующей молитве».
    pub fn new(current: u32, hard_pity: u32) -> Self {
        Self {
            current,
            hard_pity,
            remaining: hard_pity.saturating_sub(current).max(1),
        }
    }
}

/// Почему шанс получить предмет с баннера именно такой.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChanceSource {
    /// Прошлое 50/50 проиграно — следующий 5★ гарантированно с баннера.
    Guaranteed,
    /// Проиграно достаточно 50/50 подряд — сработает Capturing Radiance.
    CapturingRadiance,
    /// Обычный шанс (50/50 или итоговые 55% с Capturing Radiance).
    Base,
}

/// Статус 50/50 пула баннеров.
///
/// `#[serde(tag = "state")]` даёт в JSON поле-метку:
/// `{"state":"unknown"}` или `{"state":"known","guaranteed":false,...}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum FeaturedStatus {
    /// Определить нельзя: для части 5★ неизвестно, были ли они с баннера.
    Unknown,
    /// Статус известен.
    Known {
        /// Следующий 5★ гарантированно с баннера (прошлое 50/50 проиграно).
        guaranteed: bool,
        /// Сколько 50/50 проиграно подряд (для Capturing Radiance).
        losing_streak: u32,
        /// Сколько ещё проигрышей подряд до гарантии Capturing Radiance;
        /// `None`, если в игре нет этой механики. `0` — гарантия уже действует.
        losses_until_radiance: Option<u32>,
        /// Шанс, что следующий 5★ будет с баннера.
        next_featured_chance: Probability,
        /// Чем объясняется этот шанс.
        chance_source: ChanceSource,
    },
}

/// Исход 50/50 для конкретной 5★ молитвы (для таблицы молитв).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FiftyFiftyOutcome {
    /// 50/50 выиграно: выпал предмет с баннера.
    Won,
    /// 50/50 проиграно: выпал предмет не с баннера.
    Lost,
    /// Предмет с баннера по гаранту после проигрыша.
    Guaranteed,
    /// Нет данных о баннере.
    Unknown,
}

/// Пометки к одной молитве для таблицы истории.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct WishAnnotation {
    /// На какой по счёту молитве выпал предмет (только для 4★ и 5★).
    pub pity: Option<u32>,
    /// Исход 50/50 (для 5★ и 4★ на баннерах с соответствующим правилом).
    pub outcome: Option<FiftyFiftyOutcome>,
}

/// Итоговая статистика пула баннеров.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PoolStatistics {
    /// Всего молитв в пуле.
    pub total_wishes: u32,
    /// Счётчик гаранта 5★.
    pub five_star: PityCounter,
    /// Счётчик гаранта 4★.
    pub four_star: PityCounter,
    /// Статус 50/50 для 5★; `None`, если правило к пулу не применяется.
    pub featured: Option<FeaturedStatus>,
    /// Статус «4★ с баннера»; `None`, если правило к пулу не применяется.
    pub four_star_featured: Option<FeaturedStatus>,
}
