//! Сервис справочных данных: хранит справочники всех игр и выдаёт сводку.

use std::collections::HashMap;
use std::sync::Arc;

use domain::game::GameId;
use domain::reference::{BannerInfo, ReferenceData};
use domain::wish::ItemKind;
use serde::Serialize;
use thiserror::Error;

use super::GameCatalog;

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

/// Хранит справочники игр и отвечает на вопросы о них.
///
/// Справочники загружаются один раз при создании. На этапе 4b сюда
/// добавится замена справочника скачанным обновлением.
pub struct ReferenceService {
    references: HashMap<GameId, Arc<ReferenceData>>,
}

impl ReferenceService {
    /// Загружает встроенные справочники всех игр из каталога.
    pub fn from_catalog(catalog: &GameCatalog) -> Self {
        let references = catalog
            .modules()
            .map(|module| {
                let data = ReferenceData::new(module.embedded_reference());
                let snapshot = data.snapshot();
                tracing::info!(
                    game = %module.info().id,
                    items = snapshot.items.len(),
                    banners = snapshot.banners.len(),
                    revision = %snapshot.source_revision,
                    "справочник загружен"
                );
                (module.info().id, Arc::new(data))
            })
            .collect();
        Self { references }
    }

    /// Возвращает справочник игры.
    pub fn reference(&self, game_id: &GameId) -> Result<Arc<ReferenceData>, ReferenceError> {
        self.references
            .get(game_id)
            .cloned()
            .ok_or_else(|| ReferenceError::UnknownGame(game_id.clone()))
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
            latest_banner: latest_banner(&snapshot.banners),
        })
    }
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
fn latest_banner(banners: &[BannerInfo]) -> Option<LatestBanner> {
    banners
        .iter()
        .max_by(|left, right| left.start.time.cmp(&right.start.time))
        .map(|banner| LatestBanner {
            short_name: banner.short_name.clone(),
            version: banner.version.clone(),
            start: banner.start.time.as_str().to_owned(),
        })
}

// Тесты лежат рядом, в файле `reference_service.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "reference_service.tests.rs"]
mod tests;
