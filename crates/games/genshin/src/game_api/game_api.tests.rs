// Тесты разбора ответа API истории молитв.
use domain::import::{ApiWishPage, GameApiError, GameApiParser};
use domain::reference::{ItemInfo, LocalizedText, ReferenceData, ReferenceSnapshot};
use domain::region::ServerRegion;
use domain::wish::{ItemId, ItemKind, Rarity};

use super::GenshinApiParser;

/// Справочник с одним предметом «Ху Тао» для тестов.
fn reference_with_hutao() -> ReferenceData {
    let items = vec![ItemInfo {
        id: ItemId::new("hu_tao"),
        name: LocalizedText {
            en: "Hu Tao".into(),
            ru: "Ху Тао".into(),
        },
        rarity: Rarity::Five,
        kind: ItemKind::Character,
    }];
    ReferenceData::new(ReferenceSnapshot {
        source_revision: "test".into(),
        generated_at: "2026-01-01 00:00:00".into(),
        items,
        banners: vec![],
    })
}

/// JSON успешного ответа: одна молитва с Ху Тао.
fn success_json() -> String {
    r#"{
        "retcode": 0,
        "message": "OK",
        "data": { "list": [ {
            "id": "8765432109876543210",
            "gacha_type": "301",
            "name": "Hu Tao",
            "item_type": "Character",
            "rank_type": "5",
            "time": "2026-11-01 20:00:00"
        } ] }
    }"#
    .into()
}

#[test]
fn parse_page_разбирает_успешный_ответ() {
    let reference = reference_with_hutao();
    let page: ApiWishPage = GenshinApiParser
        .parse_page(&success_json(), &reference)
        .unwrap();

    assert_eq!(page.wishes.len(), 1);
    let wish = &page.wishes[0];
    assert_eq!(wish.id.as_str(), "8765432109876543210");
    assert_eq!(wish.banner_code.value(), 301);
    assert_eq!(wish.item_id.as_str(), "hu_tao");
    assert_eq!(wish.item_kind, ItemKind::Character);
    assert_eq!(wish.rarity, Rarity::Five);
    assert_eq!(wish.time.as_str(), "2026-11-01 20:00:00");
    assert_eq!(page.last_id.as_deref(), Some("8765432109876543210"));
}

#[test]
fn parse_page_переворачивает_порядок() {
    // Две записи: «новая» (id 2) раньше «старой» (id 1) — API отдаёт новые сверху.
    let json = r#"{
        "retcode": 0, "message": "OK",
        "data": { "list": [
            { "id": "2", "gacha_type": "301", "name": "Hu Tao", "item_type": "Character", "rank_type": "5", "time": "2026-11-02 20:00:00" },
            { "id": "1", "gacha_type": "301", "name": "Hu Tao", "item_type": "Character", "rank_type": "5", "time": "2026-11-01 20:00:00" }
        ] }
    }"#;
    let reference = reference_with_hutao();
    let page = GenshinApiParser.parse_page(json, &reference).unwrap();

    // В хронологическом порядке: старая (id 1) — первая.
    assert_eq!(page.wishes[0].id.as_str(), "1");
    assert_eq!(page.wishes[1].id.as_str(), "2");
    // last_id — последняя запись в ответе API (самая старая).
    assert_eq!(page.last_id.as_deref(), Some("1"));
}

#[test]
fn parse_page_пустой_список() {
    let json = r#"{ "retcode": 0, "message": "OK", "data": { "list": [] } }"#;
    let reference = reference_with_hutao();
    let page = GenshinApiParser.parse_page(json, &reference).unwrap();
    assert!(page.wishes.is_empty());
    assert!(page.last_id.is_none());
}

#[test]
fn parse_page_неизвестный_предмет() {
    let json = r#"{
        "retcode": 0, "message": "OK",
        "data": { "list": [ {
            "id": "3", "gacha_type": "301", "name": "Unknown Person",
            "item_type": "Character", "rank_type": "5", "time": "2026-11-01 20:00:00"
        } ] }
    }"#;
    let reference = reference_with_hutao();
    let err = GenshinApiParser.parse_page(json, &reference).unwrap_err();
    match err {
        GameApiError::UnknownItems(items) => assert_eq!(items, vec!["Unknown Person".to_string()]),
        other => panic!("ожидалась UnknownItems, получено: {other:?}"),
    }
}

#[test]
fn parse_page_ошибка_api() {
    let json = r#"{ "retcode": -100, "message": "authkey timeout", "data": null }"#;
    let reference = reference_with_hutao();
    let err = GenshinApiParser.parse_page(json, &reference).unwrap_err();
    assert!(matches!(err, GameApiError::ApiError { retcode: -100, .. }));
}

#[test]
fn parse_page_битый_json() {
    let reference = reference_with_hutao();
    let err = GenshinApiParser
        .parse_page("не json", &reference)
        .unwrap_err();
    assert!(matches!(err, GameApiError::InvalidResponse(_)));
}

#[test]
fn gacha_type_codes_содержит_все_баннеры() {
    let codes = GenshinApiParser.gacha_type_codes();
    let values: Vec<&str> = codes.iter().map(|(code, _)| code.as_str()).collect();
    for expected in ["301", "400", "302", "500", "200", "100"] {
        assert!(values.contains(&expected), "нет кода {expected}");
    }
}

#[test]
fn server_region_по_коду_из_ссылки() {
    assert_eq!(GenshinApiParser.server_region("os_usa"), Some(ServerRegion::America));
    assert_eq!(GenshinApiParser.server_region("os_euro"), Some(ServerRegion::Europe));
    assert_eq!(GenshinApiParser.server_region("os_asia"), Some(ServerRegion::Asia));
    assert_eq!(GenshinApiParser.server_region("os_cht"), Some(ServerRegion::Asia));
    assert_eq!(GenshinApiParser.server_region("cn_gf01"), Some(ServerRegion::China));
    assert_eq!(GenshinApiParser.server_region("cn_qd01"), Some(ServerRegion::China));
    assert_eq!(GenshinApiParser.server_region(""), None);
    assert_eq!(GenshinApiParser.server_region("unknown"), None);
}
