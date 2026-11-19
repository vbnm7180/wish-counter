//! Модульные тесты для `profile.rs`.
//!
//! Подключаются в `profile.rs` строками `#[cfg(test)] #[path = "profile.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn accepts_typical_uids_and_trims_spaces() {
    assert_eq!(PlayerUid::parse("700000000").unwrap().as_str(), "700000000");
    assert_eq!(
        PlayerUid::parse(" 1800000000 ").unwrap().as_str(),
        "1800000000"
    );
}

#[test]
fn rejects_non_digits() {
    assert_eq!(
        PlayerUid::parse("70000000a"),
        Err(PlayerUidError::NotDigits)
    );
    assert_eq!(
        PlayerUid::parse("-700000000"),
        Err(PlayerUidError::NotDigits)
    );
}

#[test]
fn rejects_wrong_length() {
    assert_eq!(
        PlayerUid::parse("12345"),
        Err(PlayerUidError::InvalidLength)
    );
    assert_eq!(PlayerUid::parse(""), Err(PlayerUidError::InvalidLength));
    assert_eq!(
        PlayerUid::parse("1234567890123"),
        Err(PlayerUidError::InvalidLength)
    );
}

/// Некорректный UID в JSON отклоняется при чтении.
#[test]
fn json_is_validated() {
    assert!(serde_json::from_str::<PlayerUid>("\"700000000\"").is_ok());
    assert!(serde_json::from_str::<PlayerUid>("\"abc\"").is_err());
}

#[test]
fn empty_profile_by_default() {
    let profile = GameProfile::default();
    assert_eq!(profile.server_region, None);
    assert_eq!(profile.uid, None);
}
