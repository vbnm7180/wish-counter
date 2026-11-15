//! Расчёт гаранта: проход по истории молитв одного пула.
//!
//! [`PityTracker`] получает молитвы по одной, **от старых к новым**, и ведёт
//! счётчики, как это делает игра. После прохода он выдаёт итоговую
//! статистику, а для каждой молитвы — пометки для таблицы.

use std::mem;

use super::featured::FeaturedLookup;
use super::rules::{FiftyFiftyRule, PityRules, Probability};
use super::statistics::{
    ChanceSource, FeaturedStatus, FiftyFiftyOutcome, PityCounter, PoolStatistics, WishAnnotation,
};
use crate::wish::{Rarity, Wish};

/// Состояние 50/50 во время прохода по истории.
#[derive(Debug, Clone, Copy)]
struct FiftyFiftyState {
    /// Прошлое 50/50 проиграно — следующий 5★ гарантированно с баннера.
    guaranteed: bool,
    /// Сколько 50/50 проиграно подряд.
    losing_streak: u32,
    /// Для всех 5★ известно, были ли они с баннера. Если хоть один исход
    /// неизвестен, статус дальше вычислить достоверно нельзя.
    history_complete: bool,
}

impl FiftyFiftyState {
    /// Начальное состояние: гаранта нет, проигрышей нет, история полная.
    fn new() -> Self {
        Self {
            guaranteed: false,
            losing_streak: 0,
            history_complete: true,
        }
    }

    /// Учитывает 5★ с известным результатом и возвращает исход 50/50.
    ///
    /// Гарантированный 5★ — это не 50/50, поэтому он не сбрасывает серию
    /// проигрышей: Capturing Radiance считает только сами 50/50.
    fn apply(&mut self, is_featured: bool) -> FiftyFiftyOutcome {
        match (is_featured, self.guaranteed) {
            (true, true) => {
                self.guaranteed = false;
                FiftyFiftyOutcome::Guaranteed
            }
            (true, false) => {
                self.losing_streak = 0;
                FiftyFiftyOutcome::Won
            }
            // Проигрыш при действующем гаранте в игре невозможен. Такие данные
            // считаем проигрышем, чтобы расчёт продолжился, а не упал.
            (false, _) => {
                self.guaranteed = true;
                self.losing_streak += 1;
                FiftyFiftyOutcome::Lost
            }
        }
    }
}

/// Счётчик гаранта для одного пула баннеров.
///
/// Время жизни `'a` означает: трекер хранит ссылки на правила и на
/// `FeaturedLookup` и не может «пережить» их. Так компилятор гарантирует,
/// что ссылки всегда действительны.
pub struct PityTracker<'a> {
    rules: &'a PityRules,
    featured_lookup: &'a dyn FeaturedLookup,
    total_wishes: u32,
    since_five_star: u32,
    since_four_star: u32,
    fifty_fifty: FiftyFiftyState,
}

impl<'a> PityTracker<'a> {
    /// Создаёт трекер с нулевыми счётчиками.
    pub fn new(rules: &'a PityRules, featured_lookup: &'a dyn FeaturedLookup) -> Self {
        Self {
            rules,
            featured_lookup,
            total_wishes: 0,
            since_five_star: 0,
            since_four_star: 0,
            fifty_fifty: FiftyFiftyState::new(),
        }
    }

    /// Учитывает очередную молитву и возвращает пометки для неё.
    ///
    /// Молитвы нужно передавать в хронологическом порядке.
    pub fn record(&mut self, wish: &Wish) -> WishAnnotation {
        self.total_wishes += 1;
        self.since_five_star += 1;
        self.since_four_star += 1;
        match wish.rarity {
            Rarity::Five => self.record_five_star(wish),
            Rarity::Four => self.record_four_star(),
            Rarity::Three => WishAnnotation::default(),
        }
    }

    /// Обрабатывает 5★: сбрасывает счётчик 5★ и определяет исход 50/50.
    ///
    /// Счётчик 4★ при этом **не** сбрасывается: если 5★ выпал на 10-й
    /// молитве, гарантированный 4★ переносится на 11-ю (как в игре и Paimon.moe).
    fn record_five_star(&mut self, wish: &Wish) -> WishAnnotation {
        // `mem::take` возвращает текущее значение и записывает на его место 0.
        let pity = mem::take(&mut self.since_five_star);
        WishAnnotation {
            pity: Some(pity),
            outcome: self.resolve_fifty_fifty(wish),
        }
    }

    /// Обрабатывает 4★: сбрасывает счётчик 4★.
    fn record_four_star(&mut self) -> WishAnnotation {
        WishAnnotation {
            pity: Some(mem::take(&mut self.since_four_star)),
            outcome: None,
        }
    }

    /// Определяет исход 50/50 для 5★ и обновляет состояние.
    /// Возвращает `None`, если в пуле нет правила 50/50.
    fn resolve_fifty_fifty(&mut self, wish: &Wish) -> Option<FiftyFiftyOutcome> {
        // Оператор `?` на `Option`: если правила нет, сразу вернуть `None`.
        self.rules.fifty_fifty.as_ref()?;
        let outcome = match self.featured_lookup.is_featured(wish) {
            Some(is_featured) => self.fifty_fifty.apply(is_featured),
            None => {
                self.fifty_fifty.history_complete = false;
                FiftyFiftyOutcome::Unknown
            }
        };
        Some(outcome)
    }

    /// Возвращает итоговую статистику по учтённым молитвам.
    pub fn statistics(&self) -> PoolStatistics {
        PoolStatistics {
            total_wishes: self.total_wishes,
            five_star: PityCounter::new(self.since_five_star, self.rules.five_star_hard_pity),
            four_star: PityCounter::new(self.since_four_star, self.rules.four_star_hard_pity),
            featured: self.featured_status(),
        }
    }

    /// Вычисляет статус 50/50; `None`, если в пуле нет такого правила.
    fn featured_status(&self) -> Option<FeaturedStatus> {
        let rule = self.rules.fifty_fifty.as_ref()?;
        if !self.fifty_fifty.history_complete {
            return Some(FeaturedStatus::Unknown);
        }
        let state = self.fifty_fifty;
        let (next_featured_chance, chance_source) = next_featured_chance(rule, state);
        Some(FeaturedStatus::Known {
            guaranteed: state.guaranteed,
            losing_streak: state.losing_streak,
            losses_until_radiance: rule.capturing_radiance.as_ref().map(|radiance| {
                radiance
                    .losses_for_guarantee
                    .saturating_sub(state.losing_streak)
            }),
            next_featured_chance,
            chance_source,
        })
    }
}

/// Шанс, что следующий 5★ будет с баннера, и его причина.
///
/// 1. Прошлое 50/50 проиграно → 100% (гарант).
/// 2. Проиграно N раз подряд (Capturing Radiance) → 100%.
/// 3. Иначе — итоговый шанс с Capturing Radiance (55%) или базовый (50%).
fn next_featured_chance(
    rule: &FiftyFiftyRule,
    state: FiftyFiftyState,
) -> (Probability, ChanceSource) {
    let radiance = rule.capturing_radiance.as_ref();
    if state.guaranteed {
        return (Probability::CERTAIN, ChanceSource::Guaranteed);
    }
    if radiance.is_some_and(|radiance| state.losing_streak >= radiance.losses_for_guarantee) {
        return (Probability::CERTAIN, ChanceSource::CapturingRadiance);
    }
    let base_chance = radiance.map_or(rule.base_win_chance, |radiance| {
        radiance.consolidated_win_chance
    });
    (base_chance, ChanceSource::Base)
}

/// Считает статистику пула по его молитвам (в хронологическом порядке).
///
/// `impl IntoIterator<Item = &'w Wish>` принимает и срез `&[Wish]`, и
/// отфильтрованный итератор — без копирования молитв.
pub fn calculate_pool_statistics<'w>(
    wishes: impl IntoIterator<Item = &'w Wish>,
    rules: &PityRules,
    featured_lookup: &dyn FeaturedLookup,
) -> PoolStatistics {
    let mut tracker = PityTracker::new(rules, featured_lookup);
    for wish in wishes {
        tracker.record(wish);
    }
    tracker.statistics()
}

/// Возвращает пометки (номер по гаранту, исход 50/50) для каждой молитвы
/// в том же порядке. Понадобится таблице истории молитв (этап 5).
pub fn annotate_wishes<'w>(
    wishes: impl IntoIterator<Item = &'w Wish>,
    rules: &PityRules,
    featured_lookup: &dyn FeaturedLookup,
) -> Vec<WishAnnotation> {
    let mut tracker = PityTracker::new(rules, featured_lookup);
    wishes
        .into_iter()
        .map(|wish| tracker.record(wish))
        .collect()
}

// Тесты лежат рядом, в файле `tracker.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "tracker.tests.rs"]
mod tests;
