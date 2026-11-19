//! Модульные тесты для `backup.rs`.
//!
//! Подключаются в `backup.rs` строками `#[cfg(test)] #[path = "backup.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Используется настоящий встроенный справочник Genshin.

use domain::wish::Rarity;

use super::*;
use crate::reference::embedded_reference;

/// Встроенный справочник.
fn reference() -> ReferenceData {
    ReferenceData::new(embedded_reference())
}

/// Разбирает JSON-строку.
fn parse(json: &str) -> Result<ParsedHistory, ImportError> {
    PaimonBackupParser.parse(json, &reference())
}

/// Минимальный файл: сервер, две молитвы персонажа (одна десятка) и одна стандартная.
const BACKUP: &str = r#"{
  "server": "Europe",
  "wish-uid": "700000000",
  "wish-counter-character-event": { "total": 2, "pulls": [
    { "type": "weapon", "code": "301", "id": "cool_steel", "time": "2026-11-05 12:00:00", "pity": 1 },
    { "type": "character", "code": "400", "id": "mitya", "time": "2026-11-05 12:00:00", "pity": 2, "rate": 1 }
  ] },
  "wish-counter-standard": { "pulls": [
    { "type": "weapon", "code": "200", "id": "cool_steel", "time": "2026-01-01 00:00:00", "pity": 1 }
  ] },
  "wish-counter-setting": { "firstTime": false }
}"#;

#[test]
fn reads_wishes_with_rarity_from_reference() {
    let history = parse(BACKUP).unwrap();

    assert_eq!(history.wishes.len(), 3);
    let mitya = history
        .wishes
        .iter()
        .find(|wish| wish.item_id.as_str() == "mitya")
        .unwrap();
    assert_eq!(mitya.rarity, Rarity::Five);
    assert_eq!(mitya.item_kind, ItemKind::Character);
    assert_eq!(mitya.banner_code, BannerCode::new(400));
}

#[test]
fn reads_server_region_and_uid() {
    let history = parse(BACKUP).unwrap();
    assert_eq!(history.server_region, Some(ServerRegion::Europe));
    assert_eq!(history.uid.unwrap().as_str(), "700000000");
}

/// Молитвы всех разделов возвращаются в хронологическом порядке.
#[test]
fn wishes_are_chronological() {
    let times: Vec<String> = parse(BACKUP)
        .unwrap()
        .wishes
        .iter()
        .map(|wish| wish.time.as_str().to_owned())
        .collect();

    let mut sorted = times.clone();
    sorted.sort();
    assert_eq!(times, sorted);
}

/// Идентификаторы устойчивы и различают молитвы одной секунды.
#[test]
fn synthetic_ids_are_stable_and_unique() {
    let first = parse(BACKUP).unwrap();
    let second = parse(BACKUP).unwrap();

    let ids: Vec<&str> = first.wishes.iter().map(|wish| wish.id.as_str()).collect();
    let again: Vec<&str> = second.wishes.iter().map(|wish| wish.id.as_str()).collect();
    assert_eq!(ids, again);
    assert!(ids.contains(&"paimon-301-20261105120000-0"));
    assert!(ids.contains(&"paimon-400-20261105120000-1"));
}

/// Повторно записанный кусок истории (время «пошло назад») пропускается,
/// а одинаковые молитвы одной десятки — нет.
#[test]
fn repeated_block_is_skipped_but_same_ten_pull_items_are_kept() {
    let json = r#"{ "wish-counter-weapon-event": { "pulls": [
      { "type": "weapon", "code": "302", "id": "slingshot", "time": "2026-07-06 16:24:10" },
      { "type": "weapon", "code": "302", "id": "slingshot", "time": "2026-07-06 16:24:10" },
      { "type": "weapon", "code": "302", "id": "debate_club", "time": "2026-07-06 16:45:00" },
      { "type": "weapon", "code": "302", "id": "slingshot", "time": "2026-07-06 16:24:10" },
      { "type": "weapon", "code": "302", "id": "debate_club", "time": "2026-07-06 16:45:00" },
      { "type": "weapon", "code": "302", "id": "raven_bow", "time": "2026-07-06 20:00:00" }
    ] } }"#;

    let history = parse(json).unwrap();

    assert_eq!(history.duplicates_skipped, 2);
    assert_eq!(history.wishes.len(), 4);
}

#[test]
fn unknown_items_are_reported_all_at_once() {
    let json = r#"{ "wish-counter-standard": { "pulls": [
      { "type": "character", "code": "200", "id": "future_hero", "time": "2027-01-01 00:00:00" },
      { "type": "weapon", "code": "200", "id": "future_sword", "time": "2027-01-01 00:00:01" }
    ] } }"#;

    assert_eq!(
        parse(json),
        Err(ImportError::UnknownItems(vec![
            "future_hero".into(),
            "future_sword".into()
        ]))
    );
}

#[test]
fn rejects_files_of_other_formats() {
    assert!(matches!(
        parse("not json"),
        Err(ImportError::InvalidFile { .. })
    ));
    assert!(matches!(
        parse(r#"{ "server": "Asia" }"#),
        Err(ImportError::InvalidFile { .. })
    ));
}

#[test]
fn rejects_invalid_time() {
    let json = r#"{ "wish-counter-standard": { "pulls": [
      { "type": "weapon", "code": "200", "id": "cool_steel", "time": "вчера" }
    ] } }"#;

    assert!(matches!(parse(json), Err(ImportError::InvalidFile { .. })));
}

/// Пустая история (разделы есть, молитв нет) — это не ошибка.
#[test]
fn empty_sections_are_valid() {
    let history = parse(r#"{ "wish-counter-standard": { "pulls": [] } }"#).unwrap();
    assert!(history.wishes.is_empty());
}
