//! Модульные тесты для `js_literal.rs`.
//!
//! Подключаются в `js_literal.rs` строками `#[cfg(test)] #[path = "js_literal.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

/// Разбирает `export const data = <literal>;`.
fn parse(literal: &str) -> Result<JsValue, JsParseError> {
    parse_export(&format!("export const data = {literal};"), "data")
}

#[test]
fn parses_primitives() {
    assert_eq!(parse("'text'").unwrap(), JsValue::String("text".into()));
    assert_eq!(parse("\"text\"").unwrap(), JsValue::String("text".into()));
    assert_eq!(parse("-5.5").unwrap(), JsValue::Number(-5.5));
    assert_eq!(parse("true").unwrap(), JsValue::Bool(true));
    assert_eq!(parse("null").unwrap(), JsValue::Null);
    assert_eq!(parse("undefined").unwrap(), JsValue::Null);
}

#[test]
fn parses_reference_as_name() {
    assert_eq!(
        parse("weapons.claymore").unwrap(),
        JsValue::Reference("weapons.claymore".into())
    );
}

/// Формат как в banners.js: ключи без кавычек, запятые в конце, комментарии.
#[test]
fn parses_paimon_style_object() {
    let value = parse(
        r#"{
          characters: [
            {
              name: "Ballad in Goblets", // комментарий в строке
              featured: ['venti'],
              timezoneDependent: true,
            },
            // { name: 'закомментированный баннер' },
            /* блочный
               комментарий */
          ],
          'quoted-key': 1,
        }"#,
    )
    .unwrap();

    let banner = &value.get("characters").unwrap().as_array().unwrap()[0];
    assert_eq!(
        banner.get("name").unwrap().as_str(),
        Some("Ballad in Goblets")
    );
    assert_eq!(banner.get("featured").unwrap().as_array().unwrap().len(), 1);
    assert!(banner.get("timezoneDependent").unwrap().is_true());
    assert_eq!(value.get("quoted-key").unwrap().as_f64(), Some(1.0));
    assert_eq!(
        value.get("characters").unwrap().as_array().unwrap().len(),
        1
    );
}

#[test]
fn handles_escapes_and_unicode() {
    assert_eq!(
        parse(r"'Beginners\' Wish'").unwrap().as_str(),
        Some("Beginners' Wish")
    );
    assert_eq!(parse(r"'é'").unwrap().as_str(), Some("é"));
    assert_eq!(parse("'Ху Тао'").unwrap().as_str(), Some("Ху Тао"));
}

/// Импорты перед `export const` пропускаются.
#[test]
fn finds_export_after_imports() {
    let source = "import { a } from './a';\n\nexport const characters = { aino: { rarity: 4 } };";

    let value = parse_export(source, "characters").unwrap();

    assert_eq!(
        value.get("aino").unwrap().get("rarity").unwrap().as_f64(),
        Some(4.0)
    );
}

#[test]
fn reports_missing_export() {
    assert_eq!(
        parse_export("const x = 1;", "banners"),
        Err(JsParseError::ExportNotFound("banners".into()))
    );
}

#[test]
fn reports_line_of_unexpected_character() {
    let error = parse("{\n  a: 1,\n  b: ;\n}").unwrap_err();
    assert_eq!(
        error,
        JsParseError::UnexpectedCharacter {
            character: ';',
            line: 3
        }
    );
}

#[test]
fn reports_unexpected_end() {
    assert_eq!(
        parse_export("export const data = { a: [1, 2", "data"),
        Err(JsParseError::UnexpectedEnd)
    );
}

/// Подстановки в шаблонных строках — это код, они отклоняются.
#[test]
fn rejects_template_substitution() {
    assert!(parse("`a ${b}`").is_err());
}

/// Слишком глубокая вложенность отклоняется, а не роняет программу.
#[test]
fn rejects_too_deep_nesting() {
    let deep = format!("{}{}", "[".repeat(1_000), "]".repeat(1_000));
    assert!(matches!(parse(&deep), Err(JsParseError::TooDeep { .. })));
}
