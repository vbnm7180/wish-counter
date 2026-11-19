//! Экспорт истории в формат резервной копии Paimon.moe (`paimon-moe-local-data.json`).
//!
//! Файл повторяет то, что пишет сам Paimon.moe при импорте истории
//! (`src/routes/wish/import.svelte`), поэтому восстанавливается в Paimon.moe
//! через «Settings → Import». Разделы:
//! ```json
//! "wish-counter-character-event": {
//!   "total": 1483, "legendary": 59, "rare": 8,
//!   "pulls": [ { "type": "character", "code": "301", "id": "hu_tao", "time": "...", "pity": 76, "rate": 1 } ],
//!   "guaranteed": { "legendary": false, "rare": true },
//!   "rateoff": { "legendary": { "win": 5, "lose": 7, "maxStreak": 2, "currentStreak": 0 }, "rare": { ... } }
//! }
//! ```
//! Правила Paimon.moe:
//! - `pity` у 4★/5★ — номер по гаранту, у 3★ всегда 1;
//! - `rate` (только баннеры персонажа и оружия, только 4★/5★): 0 — 50/50 проиграно,
//!   1 — выиграно, 2 — по гаранту; если сервер неизвестен, `rate` и итоги 50/50 не пишутся;
//! - `legendary`/`rare` — текущие счётчики 5★/4★.
//!
//! В файл попадают только разделы молитв, сервер и UID. Коллекция персонажей,
//! настройки и прочие разделы Paimon.moe не пишутся — при восстановлении они не затрагиваются.

use std::collections::BTreeMap;

use domain::import::{ExportError, ExportInput, HistoryExporter, HistoryFormatId};
use domain::pity::featured::{FeaturedLookup, NoFeaturedData};
use domain::pity::rules::{FiftyFiftyRule, PityRules, PoolDefinition, Probability};
use domain::pity::schedule_lookup::ScheduleFeaturedLookup;
use domain::pity::statistics::{FiftyFiftyOutcome, WishAnnotation};
use domain::pity::tracker::{annotate_wishes, calculate_pool_statistics};
use domain::region::ServerRegion;
use domain::wish::{ItemKind, Rarity, Wish};
use serde::Serialize;
use serde_json::Value;

use super::backup::PAIMON_FORMAT_ID;
use crate::pool_ids;

/// Имя файла, которое предлагается при сохранении (как у самого Paimon.moe).
const EXPORT_FILE_NAME: &str = "paimon-moe-local-data.json";
/// `pity` у 3★ в формате Paimon.moe.
const THREE_STAR_PITY: u32 = 1;
/// Шанс в правиле «с баннера» для пересчёта `rate` (значение не влияет на отметки).
const ANY_CHANCE: Probability = Probability::from_basis_points(5_000);

/// Раздел файла Paimon.moe для каждого пула и признак «считать `rate`».
const SECTIONS: &[(&str, &str, bool)] = &[
    (pool_ids::CHARACTER, "wish-counter-character-event", true),
    (pool_ids::WEAPON, "wish-counter-weapon-event", true),
    (pool_ids::STANDARD, "wish-counter-standard", false),
    (pool_ids::CHRONICLED, "wish-counter-chronicled", false),
    (pool_ids::BEGINNER, "wish-counter-beginners", false),
];

/// Одна молитва в файле. Порядок полей — как у Paimon.moe.
#[derive(Serialize)]
struct PaimonPull<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    code: String,
    id: &'a str,
    time: &'a str,
    pity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate: Option<u8>,
}

/// Признак «следующий предмет гарантированно с баннера» для 5★ и 4★.
#[derive(Serialize)]
struct Guaranteed {
    legendary: bool,
    rare: bool,
}

/// Итоги 50/50 одной редкости (имена полей — как у Paimon.moe).
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RateOff {
    win: u32,
    lose: u32,
    max_streak: u32,
    current_streak: u32,
}

impl RateOff {
    /// Учитывает исход очередной молитвы (как в Paimon.moe: гарант не меняет серию).
    fn record(&mut self, outcome: FiftyFiftyOutcome) {
        match outcome {
            FiftyFiftyOutcome::Lost => {
                self.lose += 1;
                self.max_streak = self.max_streak.max(self.current_streak);
                self.current_streak = 0;
            }
            FiftyFiftyOutcome::Won => {
                self.win += 1;
                self.current_streak += 1;
            }
            FiftyFiftyOutcome::Guaranteed | FiftyFiftyOutcome::Unknown => {}
        }
    }

    /// Завершает подсчёт: самая длинная серия учитывает и текущую.
    fn finish(mut self) -> Self {
        self.max_streak = self.max_streak.max(self.current_streak);
        self
    }
}

/// Итоги 50/50 раздела.
#[derive(Serialize)]
struct RateOffs {
    legendary: RateOff,
    rare: RateOff,
}

/// Раздел файла с молитвами одного пула.
#[derive(Serialize)]
struct PaimonSection<'a> {
    total: u32,
    legendary: u32,
    rare: u32,
    pulls: Vec<PaimonPull<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    guaranteed: Option<Guaranteed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rateoff: Option<RateOffs>,
}

/// Экспорт в формат резервной копии Paimon.moe.
#[derive(Debug, Default, Clone, Copy)]
pub struct PaimonBackupExporter;

impl HistoryExporter for PaimonBackupExporter {
    /// `paimon-moe` — тот же идентификатор, что у импорта.
    fn format_id(&self) -> HistoryFormatId {
        HistoryFormatId::new(PAIMON_FORMAT_ID)
    }

    /// `paimon-moe-local-data.json`.
    fn file_name(&self) -> String {
        EXPORT_FILE_NAME.to_owned()
    }

    /// Формирует JSON: разделы непустых пулов, сервер и UID (если известны).
    fn export(&self, input: &ExportInput<'_>) -> Result<String, ExportError> {
        let lookup: Box<dyn FeaturedLookup> = match input.profile.server_region {
            Some(region) => Box::new(ScheduleFeaturedLookup::new(
                input.reference,
                input.pools,
                region,
            )),
            None => Box::new(NoFeaturedData),
        };
        let mut root: BTreeMap<&str, Value> = BTreeMap::new();
        for &(pool_id, key, with_rate) in SECTIONS {
            let Some(pool) = input.pools.iter().find(|pool| pool.id.as_str() == pool_id) else {
                continue;
            };
            let wishes: Vec<&Wish> = input
                .wishes
                .iter()
                .filter(|wish| pool.contains(wish.banner_code))
                .collect();
            if wishes.is_empty() {
                continue;
            }
            let use_rate = with_rate && input.profile.server_region.is_some();
            let section = build_section(pool, &wishes, lookup.as_ref(), use_rate);
            root.insert(key, serde_json::to_value(section).map_err(to_export_error)?);
        }
        if let Some(region) = input.profile.server_region {
            root.insert(
                "server",
                Value::String(paimon_server_name(region).to_owned()),
            );
        }
        if let Some(uid) = &input.profile.uid {
            root.insert("wish-uid", Value::String(uid.as_str().to_owned()));
        }
        serde_json::to_string_pretty(&root).map_err(to_export_error)
    }
}

/// Собирает раздел одного пула.
fn build_section<'a>(
    pool: &PoolDefinition,
    wishes: &[&'a Wish],
    lookup: &dyn FeaturedLookup,
    use_rate: bool,
) -> PaimonSection<'a> {
    let statistics = calculate_pool_statistics(wishes.iter().copied(), &pool.rules, lookup);
    let rules = if use_rate {
        rate_rules(&pool.rules)
    } else {
        no_rate_rules(&pool.rules)
    };
    let annotations = annotate_wishes(wishes.iter().copied(), &rules, lookup);
    let mut legendary_rates = RateOff::default();
    let mut rare_rates = RateOff::default();
    let mut last_rate = (None, None);
    let pulls = wishes
        .iter()
        .zip(&annotations)
        .map(|(wish, annotation)| {
            let rate = annotation.outcome.and_then(rate_code);
            if let Some(outcome) = annotation.outcome {
                match wish.rarity {
                    Rarity::Five => {
                        legendary_rates.record(outcome);
                        last_rate.0 = rate;
                    }
                    Rarity::Four => {
                        rare_rates.record(outcome);
                        last_rate.1 = rate;
                    }
                    Rarity::Three => {}
                }
            }
            paimon_pull(wish, annotation, rate)
        })
        .collect();
    PaimonSection {
        total: statistics.total_wishes,
        legendary: statistics.five_star.current,
        rare: statistics.four_star.current,
        pulls,
        guaranteed: use_rate.then_some(Guaranteed {
            legendary: last_rate.0 == Some(0),
            rare: last_rate.1 == Some(0),
        }),
        rateoff: use_rate.then(|| RateOffs {
            legendary: legendary_rates.finish(),
            rare: rare_rates.finish(),
        }),
    }
}

/// Правила для пересчёта `rate`: «с баннера / нет» для 5★ и 4★, без Capturing Radiance
/// (Paimon.moe его не учитывает; на отметки 0/1/2 он и не влияет).
fn rate_rules(rules: &PityRules) -> PityRules {
    let rule = FiftyFiftyRule {
        base_win_chance: ANY_CHANCE,
        capturing_radiance: None,
    };
    PityRules {
        fifty_fifty: Some(rule.clone()),
        four_star_featured: Some(rule),
        ..rules.clone()
    }
}

/// Правила без отметок «с баннера» — только номера по гаранту.
fn no_rate_rules(rules: &PityRules) -> PityRules {
    PityRules {
        fifty_fifty: None,
        four_star_featured: None,
        ..rules.clone()
    }
}

/// Молитва в формате Paimon.moe.
fn paimon_pull<'a>(
    wish: &'a Wish,
    annotation: &WishAnnotation,
    rate: Option<u8>,
) -> PaimonPull<'a> {
    PaimonPull {
        kind: match wish.item_kind {
            ItemKind::Character => "character",
            ItemKind::Weapon => "weapon",
        },
        code: wish.banner_code.value().to_string(),
        id: wish.item_id.as_str(),
        time: wish.time.as_str(),
        pity: annotation.pity.unwrap_or(THREE_STAR_PITY),
        rate,
    }
}

/// Исход 50/50 → код `rate` Paimon.moe.
fn rate_code(outcome: FiftyFiftyOutcome) -> Option<u8> {
    match outcome {
        FiftyFiftyOutcome::Lost => Some(0),
        FiftyFiftyOutcome::Won => Some(1),
        FiftyFiftyOutcome::Guaranteed => Some(2),
        FiftyFiftyOutcome::Unknown => None,
    }
}

/// Название сервера так, как его пишет Paimon.moe (`src/stores/server.js`).
fn paimon_server_name(region: ServerRegion) -> &'static str {
    match region {
        ServerRegion::America => "America",
        ServerRegion::Europe => "Europe",
        ServerRegion::Asia => "Asia",
        ServerRegion::China => "China",
    }
}

/// Превращает ошибку сериализации в ошибку экспорта.
fn to_export_error(error: serde_json::Error) -> ExportError {
    ExportError(error.to_string())
}

// Тесты лежат рядом, в файле `backup_export.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "backup_export.tests.rs"]
mod tests;
