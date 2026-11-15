//! Сервис статистики молитв: счётчики гаранта и потраченные примогемы.

use std::sync::Arc;

use domain::game::{GameId, GameModule};
use domain::pity::featured::{FeaturedLookup, NoFeaturedData};
use domain::pity::rules::{PoolDefinition, PoolId};
use domain::pity::schedule_lookup::ScheduleFeaturedLookup;
use domain::pity::statistics::PoolStatistics;
use domain::pity::tracker::calculate_pool_statistics;
use domain::reference::ReferenceData;
use domain::region::ServerRegion;
use domain::wish::Wish;
use serde::Serialize;
use thiserror::Error;

use super::{GameCatalog, ReferenceService};
use crate::ports::{ProfileRepository, RepositoryError, WishRepository};

/// Ошибка расчёта статистики.
#[derive(Debug, Error)]
pub enum StatisticsError {
    /// Запрошена игра, которой нет в каталоге.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Не удалось прочитать историю молитв.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Статистика одного пула баннеров для карточки в интерфейсе.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PoolSummary {
    /// Идентификатор пула (`character`, `weapon`, ...).
    pub pool_id: PoolId,
    /// Сколько примогемов потрачено на молитвы этого пула.
    pub primogems_spent: u64,
    /// Счётчики гаранта и статус 50/50.
    ///
    /// `#[serde(flatten)]` «разворачивает» поля вложенной структуры
    /// на верхний уровень JSON: `{ "pool_id": ..., "total_wishes": ... }`.
    #[serde(flatten)]
    pub statistics: PoolStatistics,
}

/// Статистика по всей игре: итоги и карточки баннеров.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameStatistics {
    /// Игра.
    pub game_id: GameId,
    /// Сервер игрока; `None` — не выбран, исход 50/50 определить нельзя.
    pub server_region: Option<ServerRegion>,
    /// Всего молитв на всех баннерах.
    pub total_wishes: u32,
    /// Всего потрачено примогемов.
    pub primogems_spent: u64,
    /// Статистика пулов в порядке отображения.
    pub pools: Vec<PoolSummary>,
}

impl GameStatistics {
    /// Собирает статистику игры и суммирует итоги по пулам.
    fn from_pools(
        game_id: GameId,
        server_region: Option<ServerRegion>,
        pools: Vec<PoolSummary>,
    ) -> Self {
        Self {
            game_id,
            server_region,
            total_wishes: pools.iter().map(|pool| pool.statistics.total_wishes).sum(),
            primogems_spent: pools.iter().map(|pool| pool.primogems_spent).sum(),
            pools,
        }
    }
}

/// Считает статистику молитв для страницы игры.
pub struct StatisticsService {
    catalog: Arc<GameCatalog>,
    wish_repository: Arc<dyn WishRepository>,
    reference_service: Arc<ReferenceService>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl StatisticsService {
    /// Создаёт сервис. Зависимости:
    /// - каталог игр (пулы баннеров и правила);
    /// - история молитв;
    /// - справочники (расписание баннеров — для исхода 50/50);
    /// - профили игр (сервер игрока — для часового пояса).
    pub fn new(
        catalog: Arc<GameCatalog>,
        wish_repository: Arc<dyn WishRepository>,
        reference_service: Arc<ReferenceService>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            catalog,
            wish_repository,
            reference_service,
            profile_repository,
        }
    }

    /// Возвращает статистику всех пулов баннеров игры.
    pub fn game_statistics(&self, game_id: &GameId) -> Result<GameStatistics, StatisticsError> {
        let module = self.find_module(game_id)?;
        let wishes = self.wish_repository.list_wishes(game_id)?;
        let region = self.profile_repository.server_region(game_id)?;
        let reference = self
            .reference_service
            .reference(game_id)
            .map_err(|_| StatisticsError::UnknownGame(game_id.clone()))?;
        let pools = module.pools();
        let featured_lookup = build_featured_lookup(&reference, &pools, region);
        let summaries = pools
            .into_iter()
            .map(|pool| summarize_pool(pool, &wishes, featured_lookup.as_ref()))
            .collect();
        Ok(GameStatistics::from_pools(
            game_id.clone(),
            region,
            summaries,
        ))
    }

    /// Находит модуль игры в каталоге.
    fn find_module(&self, game_id: &GameId) -> Result<Arc<dyn GameModule>, StatisticsError> {
        self.catalog
            .find(game_id)
            .ok_or_else(|| StatisticsError::UnknownGame(game_id.clone()))
    }
}

/// Выбирает проверку «предмет с баннера»: по расписанию, если сервер известен,
/// иначе — заглушку (статус 50/50 будет «неизвестен», если в истории есть 5★/4★).
///
/// `Box<dyn FeaturedLookup>` — «любая реализация трейта в куче»: функция может
/// вернуть объекты разных типов через один и тот же тип результата.
fn build_featured_lookup(
    reference: &ReferenceData,
    pools: &[PoolDefinition],
    region: Option<ServerRegion>,
) -> Box<dyn FeaturedLookup> {
    match region {
        Some(region) => Box::new(ScheduleFeaturedLookup::new(reference, pools, region)),
        None => Box::new(NoFeaturedData),
    }
}

/// Считает статистику одного пула по молитвам его баннеров.
fn summarize_pool(
    pool: PoolDefinition,
    wishes: &[Wish],
    featured_lookup: &dyn FeaturedLookup,
) -> PoolSummary {
    let pool_wishes = wishes.iter().filter(|wish| pool.contains(wish.banner_code));
    let statistics = calculate_pool_statistics(pool_wishes, &pool.rules, featured_lookup);
    PoolSummary {
        primogems_spent: pool.primogems_for(statistics.total_wishes),
        pool_id: pool.id,
        statistics,
    }
}

// Тесты лежат рядом, в файле `statistics_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "statistics_service.tests.rs"]
mod tests;
