//! История молитв одного пула баннеров — строки для таблицы «Посмотреть».

use std::sync::Arc;

use domain::game::GameId;
use domain::pity::featured::{FeaturedLookup, NoFeaturedData};
use domain::pity::rules::PoolId;
use domain::pity::schedule_lookup::ScheduleFeaturedLookup;
use domain::pity::statistics::FiftyFiftyOutcome;
use domain::pity::tracker::annotate_wishes;
use domain::reference::{LocalizedText, ReferenceData};
use domain::region::ServerRegion;
use domain::wish::{ItemId, ItemKind, Rarity, Wish, WishId, WishTime};
use serde::Serialize;
use thiserror::Error;

use super::{GameCatalog, ReferenceService};
use crate::ports::{ProfileRepository, RepositoryError, WishRepository};

/// Ошибка получения истории.
#[derive(Debug, Error)]
pub enum HistoryError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// У игры нет такого пула баннеров.
    #[error("пул «{}» не найден", .0.as_str())]
    UnknownPool(PoolId),
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Одна строка таблицы молитв.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WishRow {
    /// Идентификатор молитвы (ключ строки в таблице).
    pub id: WishId,
    /// Время молитвы по серверу игры.
    pub time: WishTime,
    /// Предмет (для картинки).
    pub item_id: ItemId,
    /// Название предмета; `None`, если предмета нет в справочнике.
    pub name: Option<LocalizedText>,
    /// Персонаж или оружие.
    pub item_kind: ItemKind,
    /// Редкость.
    pub rarity: Rarity,
    /// На какой молитве по гаранту выпал предмет (только 4★ и 5★).
    pub pity: Option<u32>,
    /// Исход 50/50 (только 4★/5★ на баннерах с этим правилом).
    pub outcome: Option<FiftyFiftyOutcome>,
    /// Баннер, действовавший в момент молитвы (например, `"Mitya"`); `None` — сервер не выбран
    /// или баннер не найден.
    pub banner_name: Option<String>,
}

/// История пула: строки от новых к старым (как в игре и Paimon.moe).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PoolHistory {
    /// Пул баннеров.
    pub pool_id: PoolId,
    /// Сервер игрока; `None` — исходы 50/50 и названия баннеров неизвестны.
    pub server_region: Option<ServerRegion>,
    /// Строки таблицы, новые сверху.
    pub rows: Vec<WishRow>,
}

/// Готовит историю молитв пула для таблицы.
pub struct HistoryService {
    catalog: Arc<GameCatalog>,
    references: Arc<ReferenceService>,
    wish_repository: Arc<dyn WishRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl HistoryService {
    /// Создаёт сервис истории.
    pub fn new(
        catalog: Arc<GameCatalog>,
        references: Arc<ReferenceService>,
        wish_repository: Arc<dyn WishRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            catalog,
            references,
            wish_repository,
            profile_repository,
        }
    }

    /// Возвращает историю пула `pool_id` игры `game_id`.
    pub fn pool_history(
        &self,
        game_id: &GameId,
        pool_id: &PoolId,
    ) -> Result<PoolHistory, HistoryError> {
        let module = self
            .catalog
            .find(game_id)
            .ok_or_else(|| HistoryError::UnknownGame(game_id.clone()))?;
        let pools = module.pools();
        let pool = pools
            .iter()
            .find(|pool| &pool.id == pool_id)
            .ok_or_else(|| HistoryError::UnknownPool(pool_id.clone()))?;
        let reference = self
            .references
            .reference(game_id)
            .map_err(|_| HistoryError::UnknownGame(game_id.clone()))?;
        let region = self.profile_repository.server_region(game_id)?;
        let schedule = region.map(|region| ScheduleFeaturedLookup::new(&reference, &pools, region));
        // Без сервера — заглушка: исходы 50/50 «неизвестны».
        let lookup: &dyn FeaturedLookup = match &schedule {
            Some(schedule) => schedule,
            None => &NoFeaturedData,
        };

        let wishes = self.wish_repository.list_wishes(game_id)?;
        let pool_wishes: Vec<&Wish> = wishes
            .iter()
            .filter(|wish| pool.contains(wish.banner_code))
            .collect();
        let annotations = annotate_wishes(pool_wishes.iter().copied(), &pool.rules, lookup);
        let mut rows: Vec<WishRow> = pool_wishes
            .iter()
            .zip(annotations)
            .map(|(wish, annotation)| WishRow {
                id: wish.id.clone(),
                time: wish.time.clone(),
                item_id: wish.item_id.clone(),
                name: item_name(&reference, &wish.item_id),
                item_kind: wish.item_kind,
                rarity: wish.rarity,
                pity: annotation.pity,
                outcome: annotation.outcome,
                banner_name: schedule
                    .as_ref()
                    .and_then(|schedule| schedule.banner_name(wish))
                    .map(str::to_owned),
            })
            .collect();
        // Расчёт идёт от старых к новым, а показываем — новые сверху.
        rows.reverse();
        Ok(PoolHistory {
            pool_id: pool_id.clone(),
            server_region: region,
            rows,
        })
    }
}

/// Название предмета из справочника.
fn item_name(reference: &ReferenceData, item_id: &ItemId) -> Option<LocalizedText> {
    reference.item(item_id).map(|item| item.name.clone())
}

// Тесты лежат рядом, в файле `history_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "history_service.tests.rs"]
mod tests;
