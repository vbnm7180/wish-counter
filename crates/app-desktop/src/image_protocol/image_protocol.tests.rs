//! Модульные тесты для `image_protocol.rs`.
//!
//! Подключаются в `image_protocol.rs` строками
//! `#[cfg(test)] #[path = "image_protocol.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use super::*;

#[test]
fn parses_game_and_item() {
    let (game, item) = parse_image_path("/genshin/hu_tao").unwrap();
    assert_eq!(game.as_str(), "genshin");
    assert_eq!(item.as_str(), "hu_tao");
}

#[test]
fn accepts_dash_in_item_id() {
    assert!(parse_image_path("/genshin/primordial_jade_winged-spear").is_some());
}

#[test]
fn rejects_malformed_paths() {
    for path in [
        "",
        "/",
        "/genshin",
        "/genshin/",
        "/genshin/hu_tao/extra",
        "/genshin/../secret",
        "/genshin/%2e%2e",
        "/Genshin/hu_tao",
        "/genshin/hu tao",
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
