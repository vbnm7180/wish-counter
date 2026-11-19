//! Импорт резервной копии Paimon.moe (`paimon-moe-local-data.json`).
//!
//! Устройство файла (только нужная часть):
//! ```json
//! {
//!   "server": "Europe",
//!   "wish-uid": "700000000",
//!   "wish-counter-character-event": { "pulls": [
//!     { "type": "character", "code": "301", "id": "hu_tao", "time": "2026-11-04 18:00:00", "pity": 76, "rate": 1 }
//!   ] },
//!   "wish-counter-weapon-event": { ... }, "wish-counter-standard": { ... },
//!   "wish-counter-chronicled": { ... }, "wish-counter-beginners": { ... }
//! }
//! ```
//! Особенности формата:
//! - у молитв **нет идентификатора** от игры — он строится из баннера, времени и
//!   порядкового номера внутри одной секунды (так повторный импорт того же файла
//!   не создаёт дубликатов);
//! - **нет редкости** — она берётся из справочника по `id` предмета;
//! - `pity` и `rate` Paimon.moe считает сам — мы их не читаем, а считаем заново;
//! - в реальных файлах встречаются **повторно записанные куски истории** (время «идёт назад»,
//!   а молитвы совпадают с уже прочитанными) — они пропускаются и подсчитываются.

use std::collections::HashMap;

use domain::import::{HistoryFormatId, HistoryParser, ImportError, ParsedHistory};
use domain::profile::PlayerUid;
use domain::reference::ReferenceData;
use domain::region::ServerRegion;
use domain::wish::{BannerCode, ItemId, ItemKind, Wish, WishId, WishTime};
use serde::Deserialize;

/// Идентификатор формата.
pub const PAIMON_FORMAT_ID: &str = "paimon-moe";
/// Название формата в сообщениях об ошибках.
const FORMAT_NAME: &str = "резервную копию Paimon.moe";
/// Разделы файла с историей молитв (новичковый может отсутствовать).
const WISH_SECTIONS: &[&str] = &[
    "wish-counter-character-event",
    "wish-counter-weapon-event",
    "wish-counter-standard",
    "wish-counter-chronicled",
    "wish-counter-beginners",
];
/// Префикс искусственных идентификаторов молитв из Paimon.moe.
const SYNTHETIC_ID_PREFIX: &str = "paimon";
/// Значение поля `type` для персонажа.
const TYPE_CHARACTER: &str = "character";
/// Значение поля `type` для оружия.
const TYPE_WEAPON: &str = "weapon";

/// Корень файла: сервер и разделы с молитвами.
///
/// `#[serde(flatten)]` собирает все остальные ключи файла в `HashMap`:
/// разделы молитв берём оттуда по именам, остальное (настройки, коллекция) не нужно.
#[derive(Deserialize)]
struct PaimonBackup {
    server: Option<String>,
    #[serde(rename = "wish-uid")]
    uid: Option<String>,
    #[serde(flatten)]
    sections: HashMap<String, serde_json::Value>,
}

/// Раздел с молитвами одного типа баннеров.
#[derive(Deserialize)]
struct PaimonSection {
    #[serde(default)]
    pulls: Vec<PaimonPull>,
}

/// Одна молитва в файле Paimon.moe.
#[derive(Deserialize)]
struct PaimonPull {
    /// `character` или `weapon`.
    #[serde(rename = "type")]
    kind: String,
    /// Код баннера строкой: `"301"`.
    code: String,
    /// Идентификатор предмета.
    id: String,
    /// Время `ГГГГ-ММ-ДД чч:мм:сс`.
    time: String,
}

/// Разборщик резервной копии Paimon.moe.
#[derive(Debug, Default, Clone, Copy)]
pub struct PaimonBackupParser;

impl HistoryParser for PaimonBackupParser {
    /// `paimon-moe`.
    fn format_id(&self) -> HistoryFormatId {
        HistoryFormatId::new(PAIMON_FORMAT_ID)
    }

    /// Разбирает файл целиком; при неизвестных предметах — ошибка (ничего не импортируется).
    fn parse(
        &self,
        content: &str,
        reference: &ReferenceData,
    ) -> Result<ParsedHistory, ImportError> {
        let backup: PaimonBackup = serde_json::from_str(content).map_err(invalid)?;
        let mut wishes = Vec::new();
        let mut duplicates_skipped = 0;
        let mut unknown_items = Vec::new();
        for name in WISH_SECTIONS {
            let Some(value) = backup.sections.get(*name) else {
                continue;
            };
            let section: PaimonSection = serde_json::from_value(value.clone()).map_err(invalid)?;
            let converted = convert_section(&section.pulls, reference, &mut unknown_items)?;
            duplicates_skipped += converted.duplicates_skipped;
            wishes.extend(converted.wishes);
        }
        if !unknown_items.is_empty() {
            unknown_items.sort();
            unknown_items.dedup();
            return Err(ImportError::UnknownItems(unknown_items));
        }
        if wishes.is_empty() && !has_any_section(&backup) {
            return Err(invalid("в файле нет разделов с историей молитв"));
        }
        // Сортировка стабильная: молитвы одной секунды сохраняют порядок файла.
        wishes.sort_by(|left, right| left.time.cmp(&right.time));
        Ok(ParsedHistory {
            wishes,
            server_region: backup.server.as_deref().and_then(ServerRegion::parse),
            // Некорректный UID не мешает импорту истории — он просто не сохраняется.
            uid: backup
                .uid
                .as_deref()
                .and_then(|uid| PlayerUid::parse(uid).ok()),
            duplicates_skipped,
        })
    }
}

/// Есть ли в файле хотя бы один раздел молитв (даже пустой).
fn has_any_section(backup: &PaimonBackup) -> bool {
    WISH_SECTIONS
        .iter()
        .any(|name| backup.sections.contains_key(*name))
}

/// Молитвы одного раздела после разбора.
struct ConvertedSection {
    wishes: Vec<Wish>,
    duplicates_skipped: usize,
}

/// Ключ молитвы для поиска повторов: (баннер, время, предмет).
type PullKey<'a> = (&'a str, &'a str, &'a str);

/// Отслеживает «режим повтора»: кусок истории, записанный в файл второй раз.
///
/// Реальная история не может идти назад во времени. Если время очередной молитвы
/// раньше последнего прочитанного — начался повтор. В этом режиме пропускаются
/// молитвы, которые **по счёту** совпадают с уже прочитанными (так две одинаковые
/// 3★ из одной десятки не путаются с повтором). Режим заканчивается, когда
/// история вышла за последнее известное время или молитва не совпала.
#[derive(Default)]
struct RepeatDetector<'a> {
    /// Сколько раз каждая молитва уже принята.
    accepted: HashMap<PullKey<'a>, usize>,
    /// Сколько раз каждая молитва встретилась в текущем повторе (`None` — повтора нет).
    replayed: Option<HashMap<PullKey<'a>, usize>>,
    /// Самое позднее время среди принятых молитв.
    latest_time: &'a str,
}

impl<'a> RepeatDetector<'a> {
    /// Возвращает `true`, если молитва — повтор (её нужно пропустить).
    fn is_repeat(&mut self, key: PullKey<'a>) -> bool {
        let time = key.1;
        if time < self.latest_time && self.replayed.is_none() {
            self.replayed = Some(HashMap::new());
        }
        if let Some(replayed) = self.replayed.as_mut() {
            let used = replayed.entry(key).or_insert(0);
            if time <= self.latest_time && *used < self.accepted.get(&key).copied().unwrap_or(0) {
                *used += 1;
                return true;
            }
            self.replayed = None;
        }
        *self.accepted.entry(key).or_insert(0) += 1;
        self.latest_time = self.latest_time.max(time);
        false
    }
}

/// Переводит молитвы раздела, пропуская повторно записанные куски истории.
fn convert_section(
    pulls: &[PaimonPull],
    reference: &ReferenceData,
    unknown_items: &mut Vec<String>,
) -> Result<ConvertedSection, ImportError> {
    let mut detector = RepeatDetector::default();
    let mut ordinal_in_second: HashMap<&str, usize> = HashMap::new();
    let mut wishes = Vec::with_capacity(pulls.len());
    let mut duplicates_skipped = 0;
    for pull in pulls {
        let key = (pull.code.as_str(), pull.time.as_str(), pull.id.as_str());
        if detector.is_repeat(key) {
            duplicates_skipped += 1;
            continue;
        }
        // Порядковый номер внутри секунды (десятка молитв имеет одно время).
        let ordinal = ordinal_in_second.entry(pull.time.as_str()).or_insert(0);
        if let Some(wish) = convert_pull(pull, *ordinal, reference, unknown_items)? {
            wishes.push(wish);
        }
        *ordinal += 1;
    }
    Ok(ConvertedSection {
        wishes,
        duplicates_skipped,
    })
}

/// Переводит одну молитву. `None` — предмет неизвестен (он добавлен в `unknown_items`).
fn convert_pull(
    pull: &PaimonPull,
    ordinal: usize,
    reference: &ReferenceData,
    unknown_items: &mut Vec<String>,
) -> Result<Option<Wish>, ImportError> {
    let item_id = ItemId::new(&pull.id);
    let Some(item) = reference.item(&item_id) else {
        unknown_items.push(pull.id.clone());
        return Ok(None);
    };
    let code: u32 = pull
        .code
        .parse()
        .map_err(|_| invalid(format!("неверный код баннера «{}»", pull.code)))?;
    let time = WishTime::parse(&pull.time).map_err(invalid)?;
    Ok(Some(Wish {
        id: synthetic_id(&pull.code, &time, ordinal),
        banner_code: BannerCode::new(code),
        item_kind: parse_kind(&pull.kind).unwrap_or(item.kind),
        rarity: item.rarity,
        item_id,
        time,
    }))
}

/// Искусственный идентификатор: `paimon-301-20261104180000-0`.
/// Одинаков при каждом импорте того же файла — поэтому повторный импорт не дублирует молитвы.
fn synthetic_id(code: &str, time: &WishTime, ordinal: usize) -> WishId {
    let digits: String = time.as_str().chars().filter(char::is_ascii_digit).collect();
    WishId::new(format!("{SYNTHETIC_ID_PREFIX}-{code}-{digits}-{ordinal}"))
}

/// Тип предмета из поля `type`; `None` — неизвестное значение (тогда берём из справочника).
fn parse_kind(value: &str) -> Option<ItemKind> {
    match value {
        TYPE_CHARACTER => Some(ItemKind::Character),
        TYPE_WEAPON => Some(ItemKind::Weapon),
        _ => None,
    }
}

/// Ошибка «файл не того формата».
fn invalid(reason: impl ToString) -> ImportError {
    ImportError::InvalidFile {
        format: FORMAT_NAME.to_owned(),
        reason: reason.to_string(),
    }
}

// Тесты лежат рядом, в файле `backup.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "backup.tests.rs"]
mod tests;
