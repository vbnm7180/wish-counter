//! Сверка с Paimon.moe на настоящем файле экспорта (личные данные — в репозиторий не входит).
//!
//! Тест помечен `#[ignore]` и читает путь из переменной окружения:
//! ```text
//! WISH_COUNTER_PAIMON_EXPORT=target/paimon-export/paimon-moe-local-data.json \
//!   cargo test -p wish-counter --test paimon_export_real -- --ignored --nocapture
//! ```
//! Что проверяется:
//! - импорт проходит без ошибок, повторный импорт ничего не добавляет;
//! - номер молитвы по гаранту у каждой 4★ и 5★ совпадает с `pity` из файла;
//!   (кроме объяснимых: после повторно записанных кусков истории счётчик Paimon.moe завышен);
//! - исход 50/50 у 5★ баннера персонажа совпадает с `rate` из файла
//!   (Paimon.moe: 1 — выиграно, 0 — проиграно, 2 — по гаранту).

use std::collections::HashMap;
use std::sync::Arc;

use application::ports::WishRepository;
use application::services::{GameCatalog, ImportService, ReferenceService};
use domain::game::GameId;
use domain::import::ImportFormatId;
use domain::pity::rules::PoolDefinition;
use domain::pity::schedule_lookup::ScheduleFeaturedLookup;
use domain::pity::statistics::FiftyFiftyOutcome;
use domain::pity::tracker::annotate_wishes;
use domain::region::ServerRegion;
use domain::wish::{Rarity, Wish};
use game_genshin::paimon::backup::PAIMON_FORMAT_ID;
use game_genshin::{GENSHIN_GAME_ID, GenshinModule, pool_ids};
use infrastructure::database::{SharedConnection, open_in_memory_database};
use infrastructure::profiles::SqliteProfileRepository;
use infrastructure::wishes::SqliteWishRepository;
use serde_json::Value;

/// Переменная окружения с путём к файлу экспорта.
const EXPORT_PATH_VARIABLE: &str = "WISH_COUNTER_PAIMON_EXPORT";

/// Значения `pity` и `rate` из файла по ключу (время, предмет) — по порядку появления:
/// в одной десятке может быть несколько одинаковых предметов.
type PaimonMarks = HashMap<(String, String), Vec<(Option<u64>, Option<u64>)>>;

/// Читает отметки Paimon.moe из файла (только 4★/5★ — для них `pity` осмысленно).
fn paimon_marks(content: &str) -> PaimonMarks {
    let root: Value = serde_json::from_str(content).unwrap();
    root.as_object()
        .unwrap()
        .iter()
        .filter(|(key, _)| key.starts_with("wish-counter-"))
        .filter_map(|(_, section)| section.get("pulls")?.as_array().cloned())
        .flatten()
        .fold(PaimonMarks::new(), |mut marks, pull| {
            let key = (
                pull["time"].as_str().unwrap().to_owned(),
                pull["id"].as_str().unwrap().to_owned(),
            );
            let mark = (pull["pity"].as_u64(), pull["rate"].as_u64());
            marks.entry(key).or_default().push(mark);
            marks
        })
}

/// Перевод нашего исхода 50/50 в код `rate` Paimon.moe.
fn rate_code(outcome: FiftyFiftyOutcome) -> Option<u64> {
    match outcome {
        FiftyFiftyOutcome::Won => Some(1),
        FiftyFiftyOutcome::Lost => Some(0),
        FiftyFiftyOutcome::Guaranteed => Some(2),
        FiftyFiftyOutcome::Unknown => None,
    }
}

/// Сверяет один пул: гарант у 4★/5★ и (если есть правило) исход 50/50 у 5★.
/// Возвращает число найденных расхождений.
fn compare_pool(
    pool: &PoolDefinition,
    wishes: &[Wish],
    lookup: &ScheduleFeaturedLookup,
    marks: &PaimonMarks,
    duplicates_skipped: u64,
) -> usize {
    let pool_wishes: Vec<&Wish> = wishes
        .iter()
        .filter(|wish| pool.contains(wish.banner_code))
        .collect();
    let annotations = annotate_wishes(pool_wishes.iter().copied(), &pool.rules, lookup);
    let mut mismatches = 0;
    // Сколько отметок уже использовано по каждому ключу (одинаковые молитвы одной секунды).
    let mut used: HashMap<(String, String), usize> = HashMap::new();
    for (wish, annotation) in pool_wishes.iter().zip(&annotations) {
        if wish.rarity == Rarity::Three {
            continue;
        }
        let key = (
            wish.time.as_str().to_owned(),
            wish.item_id.as_str().to_owned(),
        );
        let occurrence = used.entry(key.clone()).or_insert(0);
        let (pity, rate) = marks[&key][*occurrence];
        *occurrence += 1;
        let ours = annotation.pity.map(u64::from);
        let pity_differs = ours != pity;
        let rate_differs = wish.rarity == Rarity::Five
            && pool.rules.fifty_fifty.is_some()
            && annotation.outcome.and_then(rate_code) != rate;
        // Paimon.moe считает повторно записанные молитвы настоящими, поэтому после
        // такого куска его счётчик может быть больше нашего — не больше числа повторов.
        let explained_by_repeats = pity_differs
            && !rate_differs
            && matches!((ours, pity), (Some(ours), Some(theirs))
                if theirs > ours && theirs - ours <= duplicates_skipped);
        if explained_by_repeats {
            println!(
                "объяснимо повторами в файле: {} {} — наш pity {ours:?}, у Paimon {pity:?}",
                pool.id.as_str(),
                wish.item_id
            );
        } else if pity_differs || rate_differs {
            mismatches += 1;
            println!(
                "расхождение {} {}: наш pity {:?} / Paimon {:?}; наш исход {:?} / Paimon rate {:?}",
                pool.id.as_str(),
                wish.item_id,
                annotation.pity,
                pity,
                annotation.outcome,
                rate
            );
        }
    }
    println!(
        "пул {}: молитв {}, проверено 4★/5★ — расхождений {mismatches}",
        pool.id.as_str(),
        pool_wishes.len()
    );
    mismatches
}

#[test]
#[ignore = "нужен личный файл экспорта Paimon.moe (WISH_COUNTER_PAIMON_EXPORT)"]
fn real_export_matches_paimon_moe() {
    let path = std::env::var(EXPORT_PATH_VARIABLE).expect("задайте WISH_COUNTER_PAIMON_EXPORT");
    let content = std::fs::read_to_string(path).unwrap();
    let genshin = GameId::new(GENSHIN_GAME_ID);

    let database = SharedConnection::new(open_in_memory_database().unwrap());
    let wishes = Arc::new(SqliteWishRepository::new(database.clone()));
    let profiles = Arc::new(SqliteProfileRepository::new(database));
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(GenshinModule)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    let import = ImportService::new(catalog, references.clone(), wishes.clone(), profiles);
    let format = ImportFormatId::new(PAIMON_FORMAT_ID);

    let summary = import.import(&genshin, &format, &content).unwrap();
    println!("импорт: {summary:?}");
    let again = import.import(&genshin, &format, &content).unwrap();
    assert_eq!(again.added, 0, "повторный импорт ничего не добавляет");

    let stored = wishes.list_wishes(&genshin).unwrap();
    let region = summary.server_region.unwrap_or(ServerRegion::Asia);
    let reference = references.reference(&genshin).unwrap();
    let pools = game_genshin::pools::genshin_pools();
    let lookup = ScheduleFeaturedLookup::new(&reference, &pools, region);
    let marks = paimon_marks(&content);

    let mismatches: usize = pools
        .iter()
        .map(|pool| {
            compare_pool(
                pool,
                &stored,
                &lookup,
                &marks,
                summary.duplicates_skipped as u64,
            )
        })
        .sum();
    assert_eq!(mismatches, 0);
    assert!(
        pools
            .iter()
            .any(|pool| pool.id.as_str() == pool_ids::CHARACTER)
    );
}
