//! Модульные тесты для `background.rs`.
//!
//! Подключаются в `background.rs` строками
//! `#[cfg(test)] #[path = "background.tests.rs"] mod tests;`.

use super::*;

#[test]
fn id_accepts_safe_strings() {
    assert_eq!(BackgroundId::parse("3f9a-01").unwrap().as_str(), "3f9a-01");
}

#[test]
fn id_rejects_unsafe_strings() {
    assert_eq!(BackgroundId::parse(""), Err(BackgroundIdError::Empty));
    assert_eq!(
        BackgroundId::parse("a".repeat(65)),
        Err(BackgroundIdError::TooLong)
    );
    for value in ["../settings", "a/b", r"a\b", "Abc", "a.png", "a b"] {
        assert_eq!(
            BackgroundId::parse(value),
            Err(BackgroundIdError::InvalidCharacters),
            "{value}"
        );
    }
}

/// Значения хэша FNV-1a сверены с опубликованными тестовыми векторами алгоритма.
#[test]
fn id_from_content_is_fnv1a_hash() {
    assert_eq!(BackgroundId::from_content(b"").as_str(), "cbf29ce484222325");
    assert_eq!(
        BackgroundId::from_content(b"a").as_str(),
        "af63dc4c8601ec8c"
    );
}

#[test]
fn same_content_gives_same_id_and_different_content_differs() {
    let first = BackgroundId::from_content(b"picture");
    assert_eq!(first, BackgroundId::from_content(b"picture"));
    assert_ne!(first, BackgroundId::from_content(b"picture2"));
    assert!(
        BackgroundId::parse(first.as_str()).is_ok(),
        "идентификатор из содержимого проходит проверку"
    );
}

#[test]
fn id_is_checked_when_read_from_json() {
    assert!(serde_json::from_str::<BackgroundId>("\"../x\"").is_err());
    let id: BackgroundId = serde_json::from_str("\"abc\"").unwrap();
    assert_eq!(serde_json::to_string(&id).unwrap(), "\"abc\"");
}

#[test]
fn formats_are_detected_by_content() {
    let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0];
    let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];
    let webp = *b"RIFF\x10\0\0\0WEBPVP8 ";

    assert_eq!(ImageFormat::detect(&png), Some(ImageFormat::Png));
    assert_eq!(ImageFormat::detect(&jpeg), Some(ImageFormat::Jpeg));
    assert_eq!(ImageFormat::detect(&webp), Some(ImageFormat::Webp));
}

#[test]
fn unsupported_content_is_not_detected() {
    assert_eq!(ImageFormat::detect(b""), None);
    assert_eq!(ImageFormat::detect(b"GIF89a"), None);
    // RIFF, но не WebP (например, WAV).
    assert_eq!(ImageFormat::detect(b"RIFF\x10\0\0\0WAVEfmt "), None);
    // RIFF короче 12 байт.
    assert_eq!(ImageFormat::detect(b"RIFF\x10\0\0\0WE"), None);
}

#[test]
fn extensions_and_mime_types_match() {
    for format in ImageFormat::ALL {
        assert_eq!(
            ImageFormat::from_extension(format.extension()),
            Some(format)
        );
        assert!(format.mime_type().starts_with("image/"));
    }
    assert_eq!(ImageFormat::from_extension("JPEG"), Some(ImageFormat::Jpeg));
    assert_eq!(ImageFormat::from_extension("PNG"), Some(ImageFormat::Png));
    assert_eq!(ImageFormat::from_extension("gif"), None);
}

#[test]
fn format_is_lowercase_in_json() {
    assert_eq!(
        serde_json::to_string(&ImageFormat::Webp).unwrap(),
        "\"webp\""
    );
}
