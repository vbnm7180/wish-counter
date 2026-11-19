//! # Модуль игры Genshin Impact
//!
//! Описывает Genshin Impact для приложения:
//! - идентификатор игры;
//! - баннеры и правила гаранта ([`pools`]): 90/80 молитв, 4★ каждые 10,
//!   50/50 и Capturing Radiance на баннере персонажа;
//! - справочник ([`reference`](mod@reference)): персонажи, оружие, расписание баннеров —
//!   встроенный снимок данных Paimon.moe;
//! - перевод данных Paimon.moe в справочник ([`paimon`]).

use domain::game::{GameId, GameInfo, GameModule};
use domain::pity::rules::PoolDefinition;
use std::sync::Arc;

use domain::reference::{ReferenceSnapshot, RemoteReference};
use paimon::remote::PaimonRemoteReference;

// Код модуля лежит в папке модуля вместе с тестами: pools/pools.rs
#[path = "pools/pools.rs"]
pub mod pools;

pub mod paimon;
// Код модуля лежит в папке модуля вместе с тестами: reference/reference.rs
#[path = "reference/reference.rs"]
pub mod reference;

pub use pools::{banner_codes, pool_ids};

/// Идентификатор Genshin Impact внутри приложения.
pub const GENSHIN_GAME_ID: &str = "genshin";

/// Иконка игры для меню (PNG 128×128, предоставлена владельцем проекта).
/// `include_bytes!` встраивает файл в программу при компиляции.
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

/// Порядок Genshin Impact в меню игр (первая игра).
const GENSHIN_DISPLAY_ORDER: u32 = 10;

/// Игровой модуль Genshin Impact.
///
/// Структура без полей (unit struct): данных у модуля пока нет,
/// но он реализует трейт [`GameModule`].
#[derive(Debug, Default, Clone, Copy)]
pub struct GenshinModule;

impl GameModule for GenshinModule {
    /// Возвращает описание Genshin Impact.
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new(GENSHIN_GAME_ID),
            display_order: GENSHIN_DISPLAY_ORDER,
        }
    }

    /// Возвращает пулы баннеров Genshin Impact.
    fn pools(&self) -> Vec<PoolDefinition> {
        pools::genshin_pools()
    }

    /// Разбирает встроенный снимок справочника (≈ миллисекунды; вызывать один раз).
    fn embedded_reference(&self) -> ReferenceSnapshot {
        reference::embedded_reference()
    }

    /// Обновления справочника берутся из репозитория Paimon.moe.
    fn remote_reference(&self) -> Option<Arc<dyn RemoteReference>> {
        Some(Arc::new(PaimonRemoteReference))
    }

    /// Встроенная иконка игры.
    fn icon_png(&self) -> Option<&'static [u8]> {
        Some(ICON_PNG)
    }
}

// Тесты лежат рядом, в файле `lib.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "lib.tests.rs"]
mod tests;
