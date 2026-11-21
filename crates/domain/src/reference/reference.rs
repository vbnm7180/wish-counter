//! Справочные данные игры: предметы (персонажи, оружие) и расписание баннеров.
//!
//! Источник данных — Paimon.moe (лицензия MIT); перевод в этот формат
//! делает модуль игры (`game_genshin::paimon`). Здесь описан сам формат,
//! общий для всех игр.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pity::rules::PoolId;
use crate::settings::Language;
use crate::wish::{ItemId, ItemKind, Rarity, WishTime};

/// Текст на поддерживаемых языках интерфейса.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedText {
    /// Английский вариант (есть всегда).
    pub en: String,
    /// Русский вариант (если перевода нет — совпадает с английским).
    pub ru: String,
}

impl LocalizedText {
    /// Возвращает текст на указанном языке.
    pub fn get(&self, language: Language) -> &str {
        match language {
            Language::English => &self.en,
            Language::Russian => &self.ru,
        }
    }
}

/// Описание предмета, который может выпасть в молитве.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemInfo {
    /// Идентификатор, например `"hu_tao"`.
    pub id: ItemId,
    /// Персонаж или оружие.
    pub kind: ItemKind,
    /// Редкость.
    pub rarity: Rarity,
    /// Название на разных языках.
    pub name: LocalizedText,
}

/// Время начала или конца баннера из расписания.
///
/// В данных Paimon.moe время записано **по азиатскому серверу** (UTC+8).
/// Баннеры, которые открываются одновременно по всему миру (сразу после
/// обновления игры), помечены `asia_based = true`: их время нужно перевести
/// в часовой пояс сервера игрока. Остальные открываются в одно и то же
/// местное время на каждом сервере — их время не пересчитывается.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleTime {
    /// Время в формате `ГГГГ-ММ-ДД чч:мм:сс`.
    pub time: WishTime,
    /// `true` — время указано по UTC+8 и требует пересчёта.
    pub asia_based: bool,
}

/// Баннер из расписания.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BannerInfo {
    /// Пул баннеров (`character`, `weapon`, ...).
    pub pool_id: PoolId,
    /// Официальное название баннера (на английском).
    pub name: String,
    /// Короткое название — обычно имя главного персонажа или оружия.
    pub short_name: String,
    /// Версия игры, например `"7.1"`.
    pub version: Option<String>,
    /// Начало действия баннера.
    pub start: ScheduleTime,
    /// Конец действия баннера.
    pub end: ScheduleTime,
    /// Предметы 5★ с повышенным шансом.
    pub featured: Vec<ItemId>,
    /// Предметы 4★ с повышенным шансом.
    pub featured_rare: Vec<ItemId>,
}

/// Справочник в том виде, в котором он хранится в JSON-файле
/// (встроенный снимок или скачанное обновление).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceSnapshot {
    /// Версия источника — идентификатор коммита репозитория Paimon.moe.
    pub source_revision: String,
    /// Когда снимок создан, `ГГГГ-ММ-ДД чч:мм:сс` (UTC).
    pub generated_at: String,
    /// Все предметы.
    pub items: Vec<ItemInfo>,
    /// Все баннеры всех пулов.
    pub banners: Vec<BannerInfo>,
}

/// Справочник, готовый к работе: снимок + индекс для быстрого поиска предмета.
///
/// Индекс (`HashMap`) строится один раз при создании, поэтому поиск предмета
/// по идентификатору занимает постоянное время, а не перебирает сотни записей.
#[derive(Debug, Clone)]
pub struct ReferenceData {
    snapshot: ReferenceSnapshot,
    item_positions: HashMap<ItemId, usize>,
    /// Индекс: английское название (нижний регистр) → позиция предмета.
    /// Нужен для импорта из API игры, который возвращает имена, а не id.
    item_positions_by_en_name: HashMap<String, usize>,
}

impl ReferenceData {
    /// Создаёт справочник из снимка и строит индексы предметов.
    pub fn new(snapshot: ReferenceSnapshot) -> Self {
        let item_positions = snapshot
            .items
            .iter()
            .enumerate()
            .map(|(position, item)| (item.id.clone(), position))
            .collect();
        let item_positions_by_en_name = snapshot
            .items
            .iter()
            .enumerate()
            .map(|(position, item)| (item.name.en.to_lowercase(), position))
            .collect();
        Self {
            snapshot,
            item_positions,
            item_positions_by_en_name,
        }
    }

    /// Находит предмет по идентификатору.
    pub fn item(&self, id: &ItemId) -> Option<&ItemInfo> {
        self.item_positions
            .get(id)
            .map(|&position| &self.snapshot.items[position])
    }

    /// Находит предмет по английскому названию (без учёта регистра).
    /// Нужен для импорта из API игры, который возвращает имена.
    pub fn item_by_en_name(&self, name: &str) -> Option<&ItemInfo> {
        self.item_positions_by_en_name
            .get(&name.to_lowercase())
            .map(|&position| &self.snapshot.items[position])
    }

    /// Возвращает баннеры указанного пула в порядке расписания.
    pub fn banners_of<'a>(&'a self, pool_id: &'a PoolId) -> impl Iterator<Item = &'a BannerInfo> {
        self.snapshot
            .banners
            .iter()
            .filter(move |banner| &banner.pool_id == pool_id)
    }

    /// Возвращает исходный снимок (например, для сохранения в файл).
    pub fn snapshot(&self) -> &ReferenceSnapshot {
        &self.snapshot
    }
}

/// Откуда скачивать обновления справочника: публичный репозиторий GitHub
/// и пути файлов внутри него.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSourceSpec {
    /// Владелец репозитория, например `"MadeBaruna"`.
    pub owner: String,
    /// Имя репозитория, например `"paimon-moe"`.
    pub repository: String,
    /// Пути нужных файлов внутри репозитория.
    pub files: Vec<String>,
}

/// Скачанные файлы: путь → содержимое.
pub type SourceFiles = HashMap<String, String>;

/// Ошибка сборки справочника из скачанных файлов.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("не удалось собрать справочник: {0}")]
pub struct ReferenceBuildError(pub String);

/// Онлайн-обновление справочника игры: где лежат исходные данные
/// и как перевести их в [`ReferenceSnapshot`].
///
/// Скачивание делает инфраструктура (она одинакова для всех игр),
/// а формат данных знает только модуль игры.
pub trait RemoteReference: Send + Sync {
    /// Откуда скачивать файлы.
    fn source(&self) -> ReferenceSourceSpec;

    /// Собирает справочник из скачанных файлов ревизии `revision`.
    fn build(
        &self,
        files: &SourceFiles,
        revision: &str,
        generated_at: &str,
    ) -> Result<ReferenceSnapshot, ReferenceBuildError>;

    /// Путь к картинке предмета внутри репозитория источника;
    /// `None`, если картинок у источника нет (так по умолчанию).
    fn image_path(&self, _item: &ItemInfo) -> Option<String> {
        None
    }
}

// Тесты лежат рядом, в файле `reference.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "reference.tests.rs"]
mod tests;
