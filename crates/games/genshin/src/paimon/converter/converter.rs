//! Перевод данных Paimon.moe в формат справочника приложения.
//!
//! На входе — тексты файлов из репозитория Paimon.moe, на выходе —
//! [`ReferenceSnapshot`]. Функции здесь ничего не скачивают и не читают
//! с диска (это делает инфраструктура), поэтому их легко тестировать.

use std::collections::{BTreeSet, HashMap};

use domain::pity::rules::PoolId;
use domain::reference::{BannerInfo, ItemInfo, LocalizedText, ReferenceSnapshot, ScheduleTime};
use domain::wish::{ItemId, ItemKind, Rarity, WishTime};
use thiserror::Error;

use super::js_literal::{JsParseError, JsValue, parse_export};
use crate::pool_ids;

/// Соответствие разделов `banners.js` пулам приложения.
const PAIMON_POOLS: &[(&str, &str)] = &[
    ("characters", pool_ids::CHARACTER),
    ("weapons", pool_ids::WEAPON),
    ("chronicled", pool_ids::CHRONICLED),
    ("standard", pool_ids::STANDARD),
    ("beginners", pool_ids::BEGINNER),
];

/// Тексты исходных файлов Paimon.moe.
#[derive(Debug, Clone)]
pub struct PaimonSources {
    /// `src/data/banners.js` — расписание баннеров.
    pub banners_js: String,
    /// `src/data/characters.js` — персонажи.
    pub characters_js: String,
    /// `src/data/weaponList.js` — оружие.
    pub weapons_js: String,
    /// `src/locales/items/ru.json` — русские названия (ключ — английское название).
    pub names_ru_json: String,
}

/// Результат перевода: справочник и найденные несоответствия.
#[derive(Debug, Clone)]
pub struct Conversion {
    /// Готовый справочник.
    pub snapshot: ReferenceSnapshot,
    /// Идентификаторы из баннеров, которых нет в списке предметов
    /// (обычно опечатка в данных Paimon.moe). Не ошибка, но стоит проверить.
    pub unknown_featured_ids: Vec<String>,
}

/// Ошибка перевода данных.
#[derive(Debug, Error)]
pub enum ConvertError {
    /// Файл не удалось разобрать.
    #[error("не удалось разобрать {file}: {source}")]
    Parse {
        /// Имя файла.
        file: &'static str,
        /// Причина.
        source: JsParseError,
    },
    /// Поле отсутствует или имеет неверный тип/формат.
    #[error("{context}: поле «{field}» отсутствует или некорректно")]
    InvalidField {
        /// Где именно (например, `banners.characters[3]`).
        context: String,
        /// Имя поля.
        field: &'static str,
    },
    /// Файл с переводами — не корректный JSON.
    #[error("файл русских названий повреждён: {0}")]
    Names(#[from] serde_json::Error),
}

/// Переводит файлы Paimon.moe в справочник.
pub fn convert(
    sources: &PaimonSources,
    source_revision: &str,
    generated_at: &str,
) -> Result<Conversion, ConvertError> {
    let russian_names: HashMap<String, String> = serde_json::from_str(&sources.names_ru_json)?;
    let characters = parse_file("characters.js", &sources.characters_js, "characters")?;
    let weapons = parse_file("weaponList.js", &sources.weapons_js, "weaponList")?;
    let banners_root = parse_file("banners.js", &sources.banners_js, "banners")?;

    let mut items = convert_items(&characters, ItemKind::Character, &russian_names)?;
    items.extend(convert_items(&weapons, ItemKind::Weapon, &russian_names)?);
    let mut banners = convert_banners(&banners_root)?;
    align_banner_ids(&items, &mut banners);
    let unknown_featured_ids = find_unknown_ids(&items, &banners);

    Ok(Conversion {
        snapshot: ReferenceSnapshot {
            source_revision: source_revision.to_owned(),
            generated_at: generated_at.to_owned(),
            items,
            banners,
        },
        unknown_featured_ids,
    })
}

/// Разбирает `export const <export_name>` из файла `file`.
fn parse_file(
    file: &'static str,
    source: &str,
    export_name: &str,
) -> Result<JsValue, ConvertError> {
    parse_export(source, export_name).map_err(|source| ConvertError::Parse { file, source })
}

// ---------- Предметы ----------

/// Переводит объект `{ id: { name, rarity, ... } }` в список предметов.
fn convert_items(
    root: &JsValue,
    kind: ItemKind,
    russian_names: &HashMap<String, String>,
) -> Result<Vec<ItemInfo>, ConvertError> {
    let entries = root.as_object().ok_or_else(|| invalid("items", "<root>"))?;
    let mut items = Vec::with_capacity(entries.len());
    for (key, value) in entries {
        if let Some(item) = convert_item(key, value, kind, russian_names)? {
            items.push(item);
        }
    }
    Ok(items)
}

/// Переводит один предмет. Возвращает `None` для оружия 1★ и 2★:
/// из молитв оно не выпадает.
fn convert_item(
    key: &str,
    value: &JsValue,
    kind: ItemKind,
    russian_names: &HashMap<String, String>,
) -> Result<Option<ItemInfo>, ConvertError> {
    let stars = required_number(value, "rarity", key)?;
    let Ok(rarity) = Rarity::try_from(stars as u8) else {
        return Ok(None);
    };
    let english = required_str(value, "name", key)?;
    let id = value.get("id").and_then(JsValue::as_str).unwrap_or(key);
    Ok(Some(ItemInfo {
        id: ItemId::new(id),
        kind,
        rarity,
        name: LocalizedText {
            en: english.to_owned(),
            ru: russian_names
                .get(english)
                .cloned()
                .unwrap_or_else(|| english.to_owned()),
        },
    }))
}

// ---------- Баннеры ----------

/// Переводит все разделы `banners.js` в один список баннеров.
fn convert_banners(root: &JsValue) -> Result<Vec<BannerInfo>, ConvertError> {
    let mut banners = Vec::new();
    for &(paimon_section, pool_id) in PAIMON_POOLS {
        let entries = root
            .get(paimon_section)
            .and_then(JsValue::as_array)
            .ok_or_else(|| invalid("banners", paimon_section))?;
        for (index, entry) in entries.iter().enumerate() {
            let context = format!("banners.{paimon_section}[{index}]");
            banners.push(convert_banner(entry, pool_id, &context)?);
        }
    }
    Ok(banners)
}

/// Переводит один баннер.
fn convert_banner(
    entry: &JsValue,
    pool_id: &str,
    context: &str,
) -> Result<BannerInfo, ConvertError> {
    let name = required_str(entry, "name", context)?;
    Ok(BannerInfo {
        pool_id: PoolId::new(pool_id),
        name: name.to_owned(),
        short_name: optional_str(entry, "shortName").unwrap_or(name).to_owned(),
        version: optional_str(entry, "version").map(str::to_owned),
        start: schedule_time(entry, "start", "timezoneDependent", context)?,
        end: schedule_time(entry, "end", "timezoneDependentEnd", context)?,
        featured: id_list(entry, "featured"),
        featured_rare: id_list(entry, "featuredRare"),
    })
}

/// Читает время начала/конца баннера и флаг «время по UTC+8».
///
/// Флаги Paimon.moe: `timezoneDependent` относится к началу баннера,
/// `timezoneDependentEnd` — к концу (так их использует сам Paimon.moe).
fn schedule_time(
    entry: &JsValue,
    field: &'static str,
    asia_flag: &str,
    context: &str,
) -> Result<ScheduleTime, ConvertError> {
    let text = required_str(entry, field, context)?;
    Ok(ScheduleTime {
        time: WishTime::parse(text).map_err(|_| invalid(context, field))?,
        asia_based: entry.get(asia_flag).is_some_and(JsValue::is_true),
    })
}

/// Читает массив идентификаторов; отсутствующее поле — пустой список.
fn id_list(entry: &JsValue, field: &str) -> Vec<ItemId> {
    entry
        .get(field)
        .and_then(JsValue::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(JsValue::as_str)
        .map(ItemId::new)
        .collect()
}

/// Приводит идентификаторы в баннерах к идентификаторам предметов,
/// если они отличаются только `-` и `_`.
///
/// В данных Paimon.moe встречаются такие расхождения: в списке оружия
/// `primordial_jade_winged-spear`, а в баннере `primordial_jade_winged_spear`.
/// `&mut` — функция изменяет баннеры на месте.
fn align_banner_ids(items: &[ItemInfo], banners: &mut [BannerInfo]) {
    let by_normalized: HashMap<String, &ItemId> = items
        .iter()
        .map(|item| (normalize_id(item.id.as_str()), &item.id))
        .collect();
    for banner in banners.iter_mut() {
        for id in banner
            .featured
            .iter_mut()
            .chain(banner.featured_rare.iter_mut())
        {
            if let Some(&item_id) = by_normalized.get(&normalize_id(id.as_str())) {
                *id = item_id.clone();
            }
        }
    }
}

/// Нормализованный вид идентификатора для сравнения: `-` заменён на `_`.
fn normalize_id(id: &str) -> String {
    id.replace('-', "_")
}

/// Собирает идентификаторы из баннеров, которых нет среди предметов.
///
/// `BTreeSet` хранит значения отсортированными и без повторов.
fn find_unknown_ids(items: &[ItemInfo], banners: &[BannerInfo]) -> Vec<String> {
    let known: BTreeSet<&str> = items.iter().map(|item| item.id.as_str()).collect();
    banners
        .iter()
        .flat_map(|banner| banner.featured.iter().chain(&banner.featured_rare))
        .map(ItemId::as_str)
        .filter(|id| !known.contains(id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

// ---------- Чтение полей ----------

/// Обязательное строковое поле.
fn required_str<'a>(
    value: &'a JsValue,
    field: &'static str,
    context: &str,
) -> Result<&'a str, ConvertError> {
    optional_str(value, field).ok_or_else(|| invalid(context, field))
}

/// Необязательное строковое поле.
fn optional_str<'a>(value: &'a JsValue, field: &str) -> Option<&'a str> {
    value.get(field).and_then(JsValue::as_str)
}

/// Обязательное числовое поле.
fn required_number(
    value: &JsValue,
    field: &'static str,
    context: &str,
) -> Result<f64, ConvertError> {
    value
        .get(field)
        .and_then(JsValue::as_f64)
        .ok_or_else(|| invalid(context, field))
}

/// Создаёт ошибку «поле отсутствует или некорректно».
fn invalid(context: &str, field: &'static str) -> ConvertError {
    ConvertError::InvalidField {
        context: context.to_owned(),
        field,
    }
}

// Тесты лежат рядом, в файле `converter.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "converter.tests.rs"]
mod tests;
