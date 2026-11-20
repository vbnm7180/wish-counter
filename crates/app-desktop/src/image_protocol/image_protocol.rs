//! Собственный протокол `wcimg` для картинок.
//!
//! Интерфейс указывает картинку обычным тегом `<img src="...">`. Адреса:
//! - `wcimg://localhost/items/<игра>/<предмет>` — картинка предмета
//!   (из кэша или скачивается при первом запросе);
//! - `wcimg://localhost/games/<игра>/icon` — иконка игры (встроена в программу);
//! - `wcimg://localhost/backgrounds/<фон>` — картинка фона, добавленная пользователем.
//!
//! В Windows WebView2 те же адреса выглядят как `http://wcimg.localhost/...`.
//!
//! Так у интерфейса нет доступа ни к файлам, ни к сети: он может получить
//! только картинку известного предмета известной игры или добавленный фон.

use std::sync::Arc;

use application::services::{BackgroundService, ImageService};
use domain::background::{BackgroundId, ImageFormat};
use domain::game::GameId;
use domain::wish::ItemId;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager, UriSchemeResponder};

use crate::state::AppState;

/// Имя протокола (схемы адреса).
pub const IMAGE_SCHEME: &str = "wcimg";
/// Сколько секунд WebView может хранить картинку у себя, не спрашивая снова (сутки).
const BROWSER_CACHE_SECONDS: u32 = 86_400;
/// Первая часть пути для картинок предметов.
const ITEMS_ROUTE: &str = "items";
/// Первая часть пути для иконок игр.
const GAMES_ROUTE: &str = "games";
/// Первая часть пути для картинок фона.
const BACKGROUNDS_ROUTE: &str = "backgrounds";
/// Последняя часть пути иконки игры.
const ICON_NAME: &str = "icon";

/// Что запрошено по адресу `wcimg://`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageRequest {
    /// Картинка предмета: `/items/<игра>/<предмет>`.
    Item {
        /// Игра.
        game_id: GameId,
        /// Предмет.
        item_id: ItemId,
    },
    /// Иконка игры: `/games/<игра>/icon`.
    GameIcon {
        /// Игра.
        game_id: GameId,
    },
    /// Картинка фона: `/backgrounds/<фон>`.
    Background {
        /// Фон.
        id: BackgroundId,
    },
}

/// Результат получения картинки: формат и байты, `None` — картинки нет.
///
/// `Box<dyn Error>` — «любая ошибка»: у сервисов разные типы ошибок, а здесь
/// нужен только текст для лога.
type LoadResult = Result<Option<(ImageFormat, Vec<u8>)>, Box<dyn std::error::Error>>;

/// Сервисы, из которых протокол берёт картинки (копии `Arc` для фонового потока).
struct ImageSources {
    /// Картинки предметов и иконки игр.
    items: Arc<ImageService>,
    /// Картинки фона.
    backgrounds: Arc<BackgroundService>,
}

/// Обрабатывает запрос картинки. Вызывается Tauri для каждого адреса `wcimg://...`.
///
/// Получение картинки может потребовать сети, поэтому работа уходит в отдельный
/// поток (`spawn_blocking`), а ответ отправляется через `responder`, когда готов.
pub fn handle_request(app: &AppHandle, request: &Request<Vec<u8>>, responder: UriSchemeResponder) {
    let Some(image_request) = parse_image_path(request.uri().path()) else {
        responder.respond(empty_response(StatusCode::BAD_REQUEST));
        return;
    };
    let state = app.state::<AppState>();
    let sources = ImageSources {
        items: Arc::clone(&state.image_service),
        backgrounds: Arc::clone(&state.background_service),
    };
    tauri::async_runtime::spawn_blocking(move || {
        responder.respond(image_response(&sources, &image_request));
    });
}

/// Получает картинку и превращает результат в HTTP-ответ.
fn image_response(sources: &ImageSources, request: &ImageRequest) -> Response<Vec<u8>> {
    match load_image(sources, request) {
        Ok(Some((format, bytes))) => image_ok_response(format, bytes),
        Ok(None) => empty_response(StatusCode::NOT_FOUND),
        Err(error) => {
            tracing::warn!(%error, "картинка не получена");
            empty_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Получает картинку и её формат. Картинки предметов и иконки игр — всегда PNG.
fn load_image(sources: &ImageSources, request: &ImageRequest) -> LoadResult {
    let png = |bytes: Option<Vec<u8>>| bytes.map(|bytes| (ImageFormat::Png, bytes));
    Ok(match request {
        ImageRequest::Item { game_id, item_id } => png(sources.items.image(game_id, item_id)?),
        ImageRequest::GameIcon { game_id } => png(sources.items.game_icon(game_id)?),
        ImageRequest::Background { id } => sources.backgrounds.image(id)?,
    })
}

/// Разбирает путь запроса. Допустимы только `/items/<игра>/<предмет>`,
/// `/games/<игра>/icon` и `/backgrounds/<фон>`; идентификаторы — только `a-z`, `0-9`, `_`, `-`
/// (у фона — без `_`, проверяет [`BackgroundId::parse`]).
pub fn parse_image_path(path: &str) -> Option<ImageRequest> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [ITEMS_ROUTE, game, item] if is_safe_segment(game) && is_safe_segment(item) => {
            Some(ImageRequest::Item {
                game_id: GameId::new(*game),
                item_id: ItemId::new(*item),
            })
        }
        [GAMES_ROUTE, game, ICON_NAME] if is_safe_segment(game) => Some(ImageRequest::GameIcon {
            game_id: GameId::new(*game),
        }),
        [BACKGROUNDS_ROUTE, id] => BackgroundId::parse(*id)
            .ok()
            .map(|id| ImageRequest::Background { id }),
        _ => None,
    }
}

/// Допустимая часть пути: не пустая, только `a-z`, `0-9`, `_`, `-`.
fn is_safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'))
}

/// Ответ с картинкой в указанном формате.
fn image_ok_response(format: ImageFormat, bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, format.mime_type())
        .header(
            header::CACHE_CONTROL,
            format!("max-age={BROWSER_CACHE_SECONDS}"),
        )
        .body(bytes)
        .unwrap_or_else(|_| empty_response(StatusCode::INTERNAL_SERVER_ERROR))
}

/// Пустой ответ с кодом ошибки.
fn empty_response(status: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = status;
    response
}

// Тесты лежат рядом, в файле `image_protocol.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "image_protocol.tests.rs"]
mod tests;
