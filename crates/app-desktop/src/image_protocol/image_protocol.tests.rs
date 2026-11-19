//! Модульные тесты для `image_protocol.rs`.
//!
//! Подключаются в `image_protocol.rs` строками
//! `#[cfg(test)] #[path = "image_protocol.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn parses_item_request() {
    assert_eq!(
        parse_image_path("/items/genshin/hu_tao"),
        Some(ImageRequest::Item {
            game_id: GameId::new("genshin"),
            item_id: ItemId::new("hu_tao"),
        })
    );
}

#[test]
fn parses_game_icon_request() {
    assert_eq!(
        parse_image_path("/games/genshin/icon"),
        Some(ImageRequest::GameIcon {
            game_id: GameId::new("genshin"),
        })
    );
}

#[test]
fn accepts_dash_in_item_id() {
    assert!(parse_image_path("/items/genshin/primordial_jade_winged-spear").is_some());
}

#[test]
fn rejects_malformed_paths() {
    for path in [
        "",
        "/",
        "/genshin/hu_tao",
        "/items/genshin",
        "/items/genshin/",
        "/items/genshin/hu_tao/extra",
        "/items/genshin/..",
        "/items/genshin/%2e%2e",
        "/items/Genshin/hu_tao",
        "/items/genshin/hu tao",
        "/games/genshin",
        "/games/genshin/logo",
        "/games/../icon",
    ] {
        assert!(parse_image_path(path).is_none(), "{path}");
    }
}

#[test]
fn png_response_has_content_type_and_cache_header() {
    let response = png_response(vec![1, 2, 3]);

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], PNG_CONTENT_TYPE);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "max-age=86400");
}
