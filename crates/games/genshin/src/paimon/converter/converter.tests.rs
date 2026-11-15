//! Модульные тесты для `converter.rs`.
//!
//! Подключаются в `converter.rs` строками `#[cfg(test)] #[path = "converter.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Входные данные — уменьшенные копии файлов Paimon.moe в том же формате.

use super::*;

/// Фрагмент `banners.js`: по одному баннеру в каждом разделе.
const BANNERS_JS: &str = r#"
export const banners = {
  beginners: [
    { name: "Beginners' Wish", start: '2000-01-01 00:00:00', end: '2200-01-01 00:00:00' },
  ],
  standard: [
    { name: 'Wanderlust Invocation', start: '2000-01-01 00:00:00', end: '2200-01-01 00:00:00' },
  ],
  characters: [
    {
      name: "Swan's Shadow in Silken Ice",
      shortName: 'Odette',
      start: '2026-08-12 06:00:00',
      end: '2026-09-01 17:59:00',
      featured: ['odette', 'arlecchino'],
      featuredRare: ['alyosha', 'sucrose', 'lynette'],
      version: '7.0',
      timezoneDependent: true,
    },
    // { name: 'закомментированный второй баннер' },
  ],
  weapons: [
    {
      name: 'Epitome Invocation',
      shortName: 'Amos',
      start: '2020-09-28 00:00:00',
      end: '2020-10-18 18:00:00',
      featured: ['amos_bow', 'typo_weapon', 'jade_spear_x'],
      timezoneDependentEnd: true,
    },
  ],
  chronicled: [],
};
"#;

/// Фрагмент `characters.js` (с импортами и ссылками, как в оригинале).
const CHARACTERS_JS: &str = r#"
import { weapons } from './weapons';
export const characters = {
  odette: { id: 'odette', name: 'Odette', rarity: 5, weapon: weapons.sword },
  sucrose: { id: 'sucrose', name: 'Sucrose', rarity: 4, weapon: weapons.catalyst },
};
"#;

/// Фрагмент `weaponList.js`: 5★, 3★ и 1★ (последнее должно быть пропущено).
const WEAPONS_JS: &str = r#"
export const weaponList = {
  amos_bow: { name: "Amos' Bow", id: 'amos_bow', rarity: 5 },
  cool_steel: { name: 'Cool Steel', id: 'cool_steel', rarity: 3 },
  'jade_spear-x': { name: 'Jade Spear', id: 'jade_spear-x', rarity: 5 },
  dull_blade: { name: 'Dull Blade', id: 'dull_blade', rarity: 1 },
};
"#;

/// Фрагмент `items/ru.json`: у Odette перевода нет.
const NAMES_RU_JSON: &str =
    r#"{ "Sucrose": "Сахароза", "Amos' Bow": "Лук Амоса", "Cool Steel": "Холодное лезвие" }"#;

/// Набор тестовых исходников.
fn sources() -> PaimonSources {
    PaimonSources {
        banners_js: BANNERS_JS.into(),
        characters_js: CHARACTERS_JS.into(),
        weapons_js: WEAPONS_JS.into(),
        names_ru_json: NAMES_RU_JSON.into(),
    }
}

/// Переводит тестовые исходники.
fn converted() -> Conversion {
    convert(&sources(), "rev", "2026-11-15 00:00:00").unwrap()
}

/// Находит предмет по идентификатору.
fn item<'a>(snapshot: &'a ReferenceSnapshot, id: &str) -> Option<&'a ItemInfo> {
    snapshot.items.iter().find(|item| item.id.as_str() == id)
}

#[test]
fn converts_characters_and_weapons() {
    let snapshot = converted().snapshot;

    let sucrose = item(&snapshot, "sucrose").unwrap();
    assert_eq!(sucrose.kind, ItemKind::Character);
    assert_eq!(sucrose.rarity, Rarity::Four);
    assert_eq!(sucrose.name.ru, "Сахароза");

    let bow = item(&snapshot, "amos_bow").unwrap();
    assert_eq!(bow.kind, ItemKind::Weapon);
    assert_eq!(bow.name.en, "Amos' Bow");
}

/// Без перевода русское название совпадает с английским.
#[test]
fn missing_translation_falls_back_to_english() {
    let snapshot = converted().snapshot;
    assert_eq!(item(&snapshot, "odette").unwrap().name.ru, "Odette");
}

/// Оружие 1★ и 2★ не выпадает из молитв и не попадает в справочник.
#[test]
fn low_rarity_weapons_are_skipped() {
    let snapshot = converted().snapshot;
    assert!(item(&snapshot, "dull_blade").is_none());
    assert_eq!(snapshot.items.len(), 5);
}

#[test]
fn converts_banners_of_all_pools() {
    let snapshot = converted().snapshot;
    let pools: Vec<&str> = snapshot
        .banners
        .iter()
        .map(|banner| banner.pool_id.as_str())
        .collect();

    // Порядок — как в PAIMON_POOLS; пустой раздел chronicled ничего не добавляет.
    assert_eq!(pools, ["character", "weapon", "standard", "beginner"]);
}

#[test]
fn converts_banner_fields_and_time_flags() {
    let snapshot = converted().snapshot;
    let odette = &snapshot.banners[0];

    assert_eq!(odette.short_name, "Odette");
    assert_eq!(odette.version.as_deref(), Some("7.0"));
    assert_eq!(odette.start.time.as_str(), "2026-08-12 06:00:00");
    assert!(odette.start.asia_based);
    assert!(!odette.end.asia_based);
    assert_eq!(odette.featured.len(), 2);
    assert_eq!(odette.featured_rare.len(), 3);

    let amos = &snapshot.banners[1];
    assert!(!amos.start.asia_based);
    assert!(amos.end.asia_based);
}

/// Без `shortName` используется полное название; без `featured` — пустой список.
#[test]
fn optional_banner_fields_have_defaults() {
    let snapshot = converted().snapshot;
    let standard = &snapshot.banners[2];

    assert_eq!(standard.short_name, "Wanderlust Invocation");
    assert!(standard.featured.is_empty());
    assert_eq!(standard.version, None);
}

/// Идентификаторы из баннеров без описания предмета попадают в отчёт.
#[test]
fn reports_unknown_featured_ids() {
    assert_eq!(
        converted().unknown_featured_ids,
        ["alyosha", "arlecchino", "lynette", "typo_weapon"]
    );
}

#[test]
fn reports_invalid_banner_time() {
    let mut broken = sources();
    broken.banners_js = broken
        .banners_js
        .replace("2026-08-12 06:00:00", "12.08.2026");

    let error = convert(&broken, "rev", "now").unwrap_err();

    assert!(
        matches!(&error, ConvertError::InvalidField { field: "start", context } if context == "banners.characters[0]"),
        "{error}"
    );
}

#[test]
fn reports_parse_errors_with_file_name() {
    let mut broken = sources();
    broken.characters_js = "export const characters = { odette: ".into();

    let error = convert(&broken, "rev", "now").unwrap_err();

    assert!(matches!(
        error,
        ConvertError::Parse {
            file: "characters.js",
            ..
        }
    ));
}

/// Идентификатор в баннере, отличающийся только `_` вместо `-`, приводится к идентификатору предмета.
#[test]
fn aligns_ids_that_differ_by_dash() {
    let snapshot = converted().snapshot;
    let amos = &snapshot.banners[1];

    assert!(amos.featured.iter().any(|id| id.as_str() == "jade_spear-x"));
}
