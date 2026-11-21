//! Описание поддерживаемых игр.
//!
//! Каждая игра (Genshin Impact, Honkai: Star Rail, ...) подключается к
//! приложению как отдельный модуль, реализующий трейт [`GameModule`].
//! Благодаря этому новая игра добавляется без изменения существующего кода
//! (принцип открытости/закрытости, буква O в SOLID).

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::import::{GameApiParser, HistoryExporter, HistoryParser};
use crate::pity::rules::PoolDefinition;
use crate::reference::{ReferenceSnapshot, RemoteReference};

/// Уникальный идентификатор игры, например `"genshin"`.
///
/// Это «обёртка» (newtype) над строкой. Она нужна, чтобы компилятор не дал
/// перепутать идентификатор игры с любой другой строкой — например,
/// с названием персонажа.
///
/// Атрибут `#[serde(transparent)]` означает, что в JSON идентификатор
/// выглядит как обычная строка: `"genshin"`, а не `{"0": "genshin"}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(String);

impl GameId {
    /// Создаёт идентификатор игры из строки.
    ///
    /// `impl Into<String>` позволяет передать и `&str`, и `String`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Возвращает идентификатор как строковый срез (без копирования).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GameId {
    /// Позволяет печатать идентификатор через `println!("{id}")` и `format!`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Краткое описание игры для меню и списка игр в интерфейсе.
///
/// Названия игр не хранятся здесь: интерфейс сам переводит их
/// на выбранный язык по идентификатору (`games.genshin.name`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameInfo {
    /// Уникальный идентификатор игры.
    pub id: GameId,
    /// Порядок отображения в меню: меньше — выше.
    pub display_order: u32,
}

/// Интерфейс (контракт) игрового модуля.
///
/// Каждая поддерживаемая игра реализует этот трейт в своём крейте
/// (`crates/games/<игра>`). На следующих этапах сюда добавятся
/// методы для импорта истории молитв.
///
/// `Send + Sync` означает, что объект можно безопасно использовать
/// из нескольких потоков: Tauri вызывает команды в разных потоках.
pub trait GameModule: Send + Sync {
    /// Возвращает описание игры.
    fn info(&self) -> GameInfo;

    /// Возвращает пулы баннеров игры в порядке отображения
    /// (у каждого пула свой счётчик гаранта и свои правила).
    fn pools(&self) -> Vec<PoolDefinition>;

    /// Возвращает справочник, встроенный в программу (предметы и баннеры).
    /// Разбор может занимать миллисекунды — вызывать один раз при запуске.
    fn embedded_reference(&self) -> ReferenceSnapshot;

    /// Возвращает описание онлайн-обновления справочника или `None`,
    /// если игра его не поддерживает (так по умолчанию).
    ///
    /// Метод с телом в трейте — **реализация по умолчанию**: игре, которой
    /// обновления не нужны, ничего писать не придётся.
    fn remote_reference(&self) -> Option<Arc<dyn RemoteReference>> {
        None
    }

    /// Иконка игры для меню (PNG, встроена в программу); `None` — иконки нет,
    /// интерфейс покажет первую букву названия.
    ///
    /// `&'static [u8]` — ссылка на байты, которые живут всё время работы программы
    /// (их встраивает `include_bytes!` при компиляции).
    fn icon_png(&self) -> Option<&'static [u8]> {
        None
    }

    /// Форматы файлов истории, которые игра умеет импортировать
    /// (по умолчанию — никаких).
    fn history_parsers(&self) -> Vec<Arc<dyn HistoryParser>> {
        Vec::new()
    }

    /// Форматы файлов истории, в которые игра умеет экспортировать
    /// (по умолчанию — никаких).
    fn history_exporters(&self) -> Vec<Arc<dyn HistoryExporter>> {
        Vec::new()
    }

    /// Разборщик ответов API игры для прямого импорта истории.
    /// `None` — игра не поддерживает прямой импорт (по умолчанию).
    fn game_api_parser(&self) -> Option<Arc<dyn GameApiParser>> {
        None
    }
}

// Тесты лежат рядом, в файле `game.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "game.tests.rs"]
mod tests;
