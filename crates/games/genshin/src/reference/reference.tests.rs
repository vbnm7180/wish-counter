//! Модульные тесты для `reference.rs`.
//!
//! Подключаются в `reference.rs` строками `#[cfg(test)] #[path = "reference.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use domain::reference::ReferenceData;
use domain::wish::ItemId;

use super::*;
use crate::pools::genshin_pools;

/// Встроенный файл читается без ошибок.
#[test]
fn embedded_reference_is_valid() {
    let snapshot = embedded_reference();
    assert!(!snapshot.source_revision.is_empty());
}

/// Справочник содержит данные, а не заглушку, и знает известных персонажей.
#[test]
fn embedded_reference_has_known_items_and_banners() {
    let data = ReferenceData::new(embedded_reference());

    let hu_tao = data
        .item(&ItemId::new("hu_tao"))
        .expect("Ху Тао должна быть в справочнике");

    assert_eq!(hu_tao.name.ru, "Ху Тао");
    assert!(data.snapshot().items.len() > 300);
    assert!(data.snapshot().banners.len() > 100);
}

/// Каждый баннер относится к одному из пулов Genshin.
#[test]
fn every_banner_belongs_to_known_pool() {
    let pools = genshin_pools();
    for banner in embedded_reference().banners {
        assert!(
            pools.iter().any(|pool| pool.id == banner.pool_id),
            "неизвестный пул {}",
            banner.pool_id.as_str()
        );
    }
}
