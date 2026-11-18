//! Онлайн-обновление справочников.
//!
//! Порядок обновления одной игры:
//! 1. если обновление не принудительное и проверка была меньше суток назад — ничего не делать;
//! 2. узнать последнюю ревизию источника (один маленький запрос);
//! 3. если она совпадает с текущей — данные актуальны;
//! 4. иначе скачать файлы, собрать справочник, проверить его, сохранить в кэш
//!    и заменить справочник в [`ReferenceService`];
//! 5. запомнить время успешной проверки.
//!
//! При любой ошибке справочник не меняется, а время проверки не записывается —
//! при следующем запуске приложение попробует снова.

use std::sync::{Arc, Mutex, PoisonError};

use chrono::{NaiveDateTime, TimeDelta};
use domain::game::GameId;
use domain::reference::{ReferenceData, ReferenceSnapshot, RemoteReference};
use serde::Serialize;
use thiserror::Error;

use super::{GameCatalog, ReferenceService, ReferenceSummary};
use crate::ports::{DownloadError, ReferenceCache, ReferenceDownloader, RepositoryError};

/// Как часто проверять обновления автоматически (в часах).
const CHECK_INTERVAL_HOURS: i64 = 24;
/// Формат времени создания справочника и времени проверки.
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
/// Во сколько раз может уменьшиться число предметов, прежде чем обновление
/// будет признано подозрительным (испорченные данные источника).
const MAX_ITEM_SHRINK_FACTOR: usize = 2;

/// Когда выполнять обновление.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    /// Только если с прошлой проверки прошло не меньше суток (автоматически при запуске).
    IfDue,
    /// Сейчас, независимо от времени прошлой проверки (кнопка «Обновить данные»).
    Force,
}

/// Результат обновления. В JSON: `{"status":"updated","revision":"..."}` и т.п.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateOutcome {
    /// Справочник обновлён до новой ревизии.
    Updated {
        /// Новая ревизия источника.
        revision: String,
    },
    /// Справочник уже актуален.
    UpToDate,
    /// Проверка не нужна: прошлая была меньше суток назад.
    NotDue,
    /// Игра не поддерживает онлайн-обновление.
    Unsupported,
}

/// Ошибка обновления.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Не удалось скачать данные (нет сети и т.п.).
    #[error(transparent)]
    Download(#[from] DownloadError),
    /// Скачанные данные не удалось разобрать или они подозрительны.
    #[error("скачанные данные некорректны: {0}")]
    InvalidData(String),
    /// Не удалось прочитать или записать кэш.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Сводка о справочнике вместе с временем последней проверки обновлений.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceStatus {
    /// Сводка о текущем справочнике.
    #[serde(flatten)]
    pub summary: ReferenceSummary,
    /// Когда последний раз успешно проверялись обновления (UTC); `None` — ни разу.
    pub last_checked_at: Option<String>,
}

/// Обновляет справочники игр из онлайн-источников.
pub struct ReferenceUpdater {
    catalog: Arc<GameCatalog>,
    references: Arc<ReferenceService>,
    cache: Arc<dyn ReferenceCache>,
    downloader: Arc<dyn ReferenceDownloader>,
    /// Не даёт двум обновлениям идти одновременно (автопроверка + кнопка).
    update_lock: Mutex<()>,
}

impl ReferenceUpdater {
    /// Создаёт сервис обновления.
    pub fn new(
        catalog: Arc<GameCatalog>,
        references: Arc<ReferenceService>,
        cache: Arc<dyn ReferenceCache>,
        downloader: Arc<dyn ReferenceDownloader>,
    ) -> Self {
        Self {
            catalog,
            references,
            cache,
            downloader,
            update_lock: Mutex::new(()),
        }
    }

    /// Обновляет справочник игры. `now` — текущее время UTC
    /// (передаётся снаружи, чтобы тесты могли задать любое время).
    pub fn update(
        &self,
        game_id: &GameId,
        mode: UpdateMode,
        now: NaiveDateTime,
    ) -> Result<UpdateOutcome, UpdateError> {
        // Охранник блокировки живёт до конца функции: второе обновление подождёт.
        let _guard = self
            .update_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let module = self
            .catalog
            .find(game_id)
            .ok_or_else(|| UpdateError::UnknownGame(game_id.clone()))?;
        let Some(remote) = module.remote_reference() else {
            return Ok(UpdateOutcome::Unsupported);
        };
        if mode == UpdateMode::IfDue && !self.is_due(game_id, now)? {
            return Ok(UpdateOutcome::NotDue);
        }
        let outcome = self.fetch_if_changed(game_id, remote.as_ref(), now)?;
        self.cache.set_last_checked_at(game_id, now)?;
        tracing::info!(game = %game_id, ?outcome, "проверка обновления справочника");
        Ok(outcome)
    }

    /// Обновляет все игры, для которых подошло время проверки.
    /// Ошибки не прерывают обновление остальных игр.
    pub fn update_all_due(
        &self,
        now: NaiveDateTime,
    ) -> Vec<(GameId, Result<UpdateOutcome, UpdateError>)> {
        self.catalog
            .modules()
            .map(|module| {
                let game_id = module.info().id;
                let result = self.update(&game_id, UpdateMode::IfDue, now);
                if let Err(error) = &result {
                    tracing::warn!(game = %game_id, %error, "справочник не обновлён");
                }
                (game_id, result)
            })
            .collect()
    }

    /// Возвращает сводку о справочнике и время последней проверки.
    pub fn status(&self, game_id: &GameId) -> Result<ReferenceStatus, UpdateError> {
        let summary = self
            .references
            .summary(game_id)
            .map_err(|_| UpdateError::UnknownGame(game_id.clone()))?;
        let last_checked_at = self
            .cache
            .last_checked_at(game_id)?
            .map(|at| at.format(TIMESTAMP_FORMAT).to_string());
        Ok(ReferenceStatus {
            summary,
            last_checked_at,
        })
    }

    /// Пора ли проверять обновление: прошли сутки или проверок не было.
    fn is_due(&self, game_id: &GameId, now: NaiveDateTime) -> Result<bool, UpdateError> {
        Ok(match self.cache.last_checked_at(game_id)? {
            Some(checked_at) => now - checked_at >= TimeDelta::hours(CHECK_INTERVAL_HOURS),
            None => true,
        })
    }

    /// Скачивает и применяет новую ревизию, если она отличается от текущей.
    fn fetch_if_changed(
        &self,
        game_id: &GameId,
        remote: &dyn RemoteReference,
        now: NaiveDateTime,
    ) -> Result<UpdateOutcome, UpdateError> {
        let source = remote.source();
        let latest = self.downloader.latest_revision(&source)?;
        let current = self
            .references
            .reference(game_id)
            .map_err(|_| UpdateError::UnknownGame(game_id.clone()))?;
        if current.snapshot().source_revision == latest {
            return Ok(UpdateOutcome::UpToDate);
        }
        let files = self.downloader.download(&source, &latest)?;
        let generated_at = now.format(TIMESTAMP_FORMAT).to_string();
        let snapshot = remote
            .build(&files, &latest, &generated_at)
            .map_err(|error| UpdateError::InvalidData(error.0))?;
        validate(&snapshot, current.snapshot())?;
        self.cache.save_snapshot(game_id, &snapshot)?;
        self.references
            .replace(game_id, ReferenceData::new(snapshot));
        Ok(UpdateOutcome::Updated { revision: latest })
    }
}

/// Проверяет, что новый справочник правдоподобен: не пустой и не уменьшился
/// резко по сравнению с текущим (защита от испорченных данных источника).
fn validate(new: &ReferenceSnapshot, current: &ReferenceSnapshot) -> Result<(), UpdateError> {
    if new.items.is_empty() || new.banners.is_empty() {
        return Err(UpdateError::InvalidData("справочник пуст".into()));
    }
    if new.items.len() * MAX_ITEM_SHRINK_FACTOR < current.items.len() {
        return Err(UpdateError::InvalidData(format!(
            "предметов стало подозрительно мало: {} вместо {}",
            new.items.len(),
            current.items.len()
        )));
    }
    Ok(())
}

// Тесты лежат рядом, в файле `reference_updater.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "reference_updater.tests.rs"]
mod tests;
