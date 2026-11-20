//! Модульные тесты для `settings.rs`.
//!
//! Подключаются в `settings.rs` строками `#[cfg(test)] #[path = "settings.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn theme_id_accepts_valid_value() {
    assert!(ThemeId::parse("high-contrast-2").is_ok());
}

#[test]
fn theme_id_rejects_empty_value() {
    assert_eq!(ThemeId::parse(""), Err(ThemeIdError::Empty));
}

#[test]
fn theme_id_rejects_too_long_value() {
    let long = "a".repeat(MAX_THEME_ID_LENGTH + 1);
    assert_eq!(ThemeId::parse(long), Err(ThemeIdError::TooLong));
}

#[test]
fn theme_id_rejects_unsafe_characters() {
    assert_eq!(
        ThemeId::parse("dark\"><script>"),
        Err(ThemeIdError::InvalidCharacters)
    );
}

/// Настройки превращаются в ожидаемый JSON и читаются обратно без потерь.
#[test]
fn settings_roundtrip_through_json() {
    let settings = AppSettings {
        theme: ThemeId::parse("light").unwrap(),
        language: Language::English,
        background: Some(BackgroundId::parse("abc123").unwrap()),
    };

    let json = serde_json::to_string(&settings).unwrap();
    let restored: AppSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(
        json,
        r#"{"theme":"light","language":"en","background":"abc123"}"#
    );
    assert_eq!(restored, settings);
}

/// Отсутствующие поля заполняются значениями по умолчанию.
#[test]
fn missing_fields_use_defaults() {
    let restored: AppSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(restored, AppSettings::default());
}

/// Некорректная тема в JSON приводит к ошибке чтения, а не к «мусору».
#[test]
fn invalid_theme_in_json_is_rejected() {
    let result = serde_json::from_str::<AppSettings>(r#"{"theme":"<bad>"}"#);
    assert!(result.is_err());
}

/// Файл настроек до этапа 5c (без поля `background`) читается: фона нет.
#[test]
fn settings_without_background_have_no_background() {
    let restored: AppSettings =
        serde_json::from_str(r#"{"theme":"dark","language":"ru"}"#).unwrap();
    assert_eq!(restored.background, None);
}

/// Небезопасный идентификатор фона в JSON отклоняется.
#[test]
fn invalid_background_in_json_is_rejected() {
    let result = serde_json::from_str::<AppSettings>(r#"{"background":"../x"}"#);
    assert!(result.is_err());
}
