//! Определение «предмет с баннера или нет» по расписанию баннеров.
//!
//! Для молитвы находится баннер её пула, который действовал в момент молитвы
//! (с учётом часового пояса сервера), и проверяется, есть ли выпавший предмет
//! в списке предметов с повышенным шансом: `featured` для 5★, `featured_rare` для 4★.
//! Логика повторяет Paimon.moe (`src/functions/wish.js`), чтобы результаты совпадали.

use std::collections::{HashMap, HashSet};

use chrono::{Duration, NaiveDateTime};

use super::featured::FeaturedLookup;
use super::rules::{PoolDefinition, PoolId};
use crate::reference::{BannerInfo, ReferenceData, ScheduleTime};
use crate::region::ServerRegion;
use crate::wish::{BannerCode, ItemId, Rarity, Wish};

/// Запас после конца баннера. В данных Paimon.moe конец часто записан
/// как `17:59:00`, а молитва в `17:59:30` ещё относится к этому баннеру.
/// Следующий баннер начинается не раньше `18:00:00`, поэтому пересечений нет.
const END_GRACE: Duration = Duration::seconds(60);

/// Баннер, подготовленный для быстрой проверки: время уже в поясе сервера.
struct BannerWindow {
    start: NaiveDateTime,
    end: NaiveDateTime,
    featured: HashSet<ItemId>,
    featured_rare: HashSet<ItemId>,
}

impl BannerWindow {
    /// Готовит баннер для сервера `region`.
    fn new(banner: &BannerInfo, region: ServerRegion) -> Self {
        Self {
            start: to_server_time(&banner.start, region),
            end: to_server_time(&banner.end, region) + END_GRACE,
            featured: banner.featured.iter().cloned().collect(),
            featured_rare: banner.featured_rare.iter().cloned().collect(),
        }
    }

    /// Действовал ли баннер в момент `time` (начало включительно, конец — нет).
    fn contains(&self, time: NaiveDateTime) -> bool {
        self.start <= time && time < self.end
    }

    /// Есть ли предмет среди предметов с повышенным шансом для его редкости.
    /// `None` для 3★: у них нет понятия «с баннера».
    fn is_featured(&self, wish: &Wish) -> Option<bool> {
        match wish.rarity {
            Rarity::Five => Some(self.featured.contains(&wish.item_id)),
            Rarity::Four => Some(self.featured_rare.contains(&wish.item_id)),
            Rarity::Three => None,
        }
    }
}

/// Переводит время из расписания в часовой пояс сервера.
///
/// Время, помеченное `asia_based`, указано по UTC+8 — сдвигаем его назад
/// на разницу с сервером (для Америки — на 13 часов). Остальное время
/// уже местное для каждого сервера.
fn to_server_time(schedule: &ScheduleTime, region: ServerRegion) -> NaiveDateTime {
    let time = schedule.time.to_date_time();
    if schedule.asia_based {
        time - Duration::hours(region.hours_behind_asia())
    } else {
        time
    }
}

/// Реализация [`FeaturedLookup`] по расписанию баннеров.
pub struct ScheduleFeaturedLookup {
    pool_by_code: HashMap<BannerCode, PoolId>,
    windows_by_pool: HashMap<PoolId, Vec<BannerWindow>>,
}

impl ScheduleFeaturedLookup {
    /// Готовит проверку для пулов игры, справочника и сервера игрока.
    pub fn new(reference: &ReferenceData, pools: &[PoolDefinition], region: ServerRegion) -> Self {
        let pool_by_code = pools
            .iter()
            .flat_map(|pool| {
                pool.banner_codes
                    .iter()
                    .map(|&code| (code, pool.id.clone()))
            })
            .collect();
        let windows_by_pool = pools
            .iter()
            .map(|pool| {
                let windows = reference
                    .banners_of(&pool.id)
                    .map(|banner| BannerWindow::new(banner, region))
                    .collect();
                (pool.id.clone(), windows)
            })
            .collect();
        Self {
            pool_by_code,
            windows_by_pool,
        }
    }

    /// Находит баннер пула молитвы, действовавший в момент молитвы.
    fn window_for(&self, wish: &Wish) -> Option<&BannerWindow> {
        let pool_id = self.pool_by_code.get(&wish.banner_code)?;
        let time = wish.time.to_date_time();
        self.windows_by_pool
            .get(pool_id)?
            .iter()
            .find(|window| window.contains(time))
    }
}

impl FeaturedLookup for ScheduleFeaturedLookup {
    /// `None`, если баннер не найден (например, справочник устарел).
    fn is_featured(&self, wish: &Wish) -> Option<bool> {
        self.window_for(wish)?.is_featured(wish)
    }
}

// Тесты лежат рядом, в файле `schedule_lookup.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "schedule_lookup.tests.rs"]
mod tests;
