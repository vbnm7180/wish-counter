//! Сервис справочных данных: хранит справочники всех игр и выдаёт сводку.

use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};

use domain::game::{GameId, GameModule};
use domain::reference::{LocalizedText, ReferenceData, ReferenceSnapshot};
use domain::wish::{ItemId, ItemKind};
use serde::Serialize;
use thiserror::Error;

use super::GameCatalog;
use crate::ports::ReferenceCache;

/// Ошибка сервиса справочников.
#[derive(Debug, Error)]
pub enum ReferenceError {
    /// Для игры нет справочника (игра не зарегистрирована).
    #[error("нет справочника для игры «{0}»")]
    UnknownGame(GameId),
}

/// Последний (самый новый) баннер из справочника.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LatestBanner {
    /// Короткое название (обычно имя персонажа).
    pub short_name: String,
    /// Версия игры.
    pub version: Option<String>,
    /// Начало баннера, `ГГГГ-ММ-ДД чч:мм:сс`.
    pub start: String,
    /// Предметы 5★ с повышенным шансом (с названиями).
    pub featured: Vec<FeaturedItem>,
}

/// Предмет баннера для отображения: идентификатор (для картинки) и название.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeaturedItem {
    /// Идентификатор предмета.
    pub id: ItemId,
    /// Название на разных языках; `None`, если предмета нет в справочнике.
    pub name: Option<LocalizedText>,
}

/// Сводка о справочнике для страницы настроек.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceSummary {
    /// Игра.
    pub game_id: GameId,
    /// Ревизия источника (коммит Paimon.moe).
    pub source_revision: String,
    /// Когда создан снимок (UTC).
    pub generated_at: String,
    /// Количество персонажей.
    pub character_count: usize,
    /// Количество видов оружия.
    pub weapon_count: usize,
    /// Количество баннеров.
    pub banner_count: usize,
    /// Самый новый баннер.
    pub latest_banner: Option<LatestBanner>,
}

/// Хранит актуальные справочники игр и отвечает на вопросы о них.
///
/// Справочник игры можно заменить во время работы ([`ReferenceService::replace`]) —
/// так применяется скачанное обновление. Поэтому таблица справочников
/// защищена `RwLock`: читать могут сразу несколько потоков, а запись
/// ждёт, пока чтение закончится.
pub struct ReferenceService {
    references: RwLock<HashMap<GameId, Arc<ReferenceData>>>,
}

impl ReferenceService {
    /// Загружает справочники всех игр: встроенный или скачанный — какой новее.
    ///
    /// Ошибка чтения скачанного справочника не мешает запуску: используется встроенный.
    pub fn new(catalog: &GameCatalog, cache: &dyn ReferenceCache) -> Self {
        Self::build(catalog, |module| {
            let embedded = module.embedded_reference();
            match cache.load_snapshot(&module.info().id) {
                Ok(Some(downloaded)) => newer(embedded, downloaded),
                Ok(None) => embedded,
                Err(error) => {
                    tracing::warn!(%error, "скачанный справочник не прочитан, используется встроенный");
                    embedded
                }
            }
        })
    }

    /// Загружает только встроенные справочники (без скачанных обновлений).
    pub fn from_catalog(catalog: &GameCatalog) -> Self {
        Self::build(catalog, |module| module.embedded_reference())
    }

    /// Строит таблицу справочников, получая снимок каждой игры функцией `snapshot_of`.
    fn build(
        catalog: &GameCatalog,
        snapshot_of: impl Fn(&dyn GameModule) -> ReferenceSnapshot,
    ) -> Self {
        let references = catalog
            .modules()
            .map(|module| {
                let data = ReferenceData::new(snapshot_of(module.as_ref()));
                log_loaded(&module.info().id, &data);
                (module.info().id, Arc::new(data))
            })
            .collect();
        Self {
            references: RwLock::new(references),
        }
    }

    /// Возвращает справочник игры.
    pub fn reference(&self, game_id: &GameId) -> Result<Arc<ReferenceData>, ReferenceError> {
        // `RwLock` «отравляется», если поток упал во время записи. Таблица всё равно
        // целая (запись — одна вставка `Arc`), поэтому берём данные как есть.
        self.references
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(game_id)
            .cloned()
            .ok_or_else(|| ReferenceError::UnknownGame(game_id.clone()))
    }

    /// Заменяет справочник игры (после успешного обновления).
    ///
    /// Те, кто уже получил старый `Arc<ReferenceData>`, дорабатывают со старыми
    /// данными; новые запросы получают новый справочник.
    pub fn replace(&self, game_id: &GameId, data: ReferenceData) {
        log_loaded(game_id, &data);
        self.references
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(game_id.clone(), Arc::new(data));
    }

    /// Возвращает сводку о справочнике игры.
    pub fn summary(&self, game_id: &GameId) -> Result<ReferenceSummary, ReferenceError> {
        let reference = self.reference(game_id)?;
        let snapshot = reference.snapshot();
        Ok(ReferenceSummary {
            game_id: game_id.clone(),
            source_revision: snapshot.source_revision.clone(),
            generated_at: snapshot.generated_at.clone(),
            character_count: count_items(&reference, ItemKind::Character),
            weapon_count: count_items(&reference, ItemKind::Weapon),
            banner_count: snapshot.banners.len(),
            latest_banner: latest_banner(&reference),
        })
    }
}

/// Выбирает более новый снимок по времени создания.
/// Строки `ГГГГ-ММ-ДД чч:мм:сс` сравниваются в хронологическом порядке.
fn newer(embedded: ReferenceSnapshot, downloaded: ReferenceSnapshot) -> ReferenceSnapshot {
    if downloaded.generated_at > embedded.generated_at {
        downloaded
    } else {
        embedded
    }
}

/// Пишет в лог, какой справочник загружен.
fn log_loaded(game_id: &GameId, data: &ReferenceData) {
    let snapshot = data.snapshot();
    tracing::info!(
        game = %game_id,
        items = snapshot.items.len(),
        banners = snapshot.banners.len(),
        revision = %snapshot.source_revision,
        generated_at = %snapshot.generated_at,
        "справочник загружен"
    );
}

/// Считает предметы указанного типа.
fn count_items(reference: &ReferenceData, kind: ItemKind) -> usize {
    reference
        .snapshot()
        .items
        .iter()
        .filter(|item| item.kind == kind)
        .count()
}

/// Находит баннер с самым поздним началом.
fn latest_banner(reference: &ReferenceData) -> Option<LatestBanner> {
    reference
        .snapshot()
        .banners
        .iter()
        .max_by(|left, right| left.start.time.cmp(&right.start.time))
        .map(|banner| LatestBanner {
            short_name: banner.short_name.clone(),
            version: banner.version.clone(),
            start: banner.start.time.as_str().to_owned(),
            featured: banner
                .featured
                .iter()
                .map(|id| FeaturedItem {
                    id: id.clone(),
                    name: reference.item(id).map(|item| item.name.clone()),
                })
                .collect(),
        })
}

// Тесты лежат рядом, в файле `reference_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "reference_service.tests.rs"]
mod tests;
