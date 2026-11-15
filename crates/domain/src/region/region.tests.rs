//! Модульные тесты для `region.rs`.
//!
//! Подключаются в `region.rs` строками `#[cfg(test)] #[path = "region.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn offsets_match_paimon_moe() {
    assert_eq!(ServerRegion::America.utc_offset_hours(), -5);
    assert_eq!(ServerRegion::Europe.utc_offset_hours(), 1);
    assert_eq!(ServerRegion::Asia.utc_offset_hours(), 8);
    assert_eq!(ServerRegion::China.utc_offset_hours(), 8);
}

#[test]
fn hours_behind_asia() {
    assert_eq!(ServerRegion::America.hours_behind_asia(), 13);
    assert_eq!(ServerRegion::Europe.hours_behind_asia(), 7);
    assert_eq!(ServerRegion::Asia.hours_behind_asia(), 0);
}

/// Названия из файлов Paimon.moe («America») распознаются без учёта регистра.
#[test]
fn parses_paimon_names_case_insensitively() {
    assert_eq!(ServerRegion::parse("America"), Some(ServerRegion::America));
    assert_eq!(ServerRegion::parse("europe"), Some(ServerRegion::Europe));
    assert_eq!(ServerRegion::parse("Mars"), None);
}

#[test]
fn serializes_as_lowercase_string() {
    assert_eq!(
        serde_json::to_string(&ServerRegion::Asia).unwrap(),
        "\"asia\""
    );
    for region in ServerRegion::ALL {
        let json = serde_json::to_string(&region).unwrap();
        assert_eq!(json.trim_matches('"'), region.as_str());
    }
}
