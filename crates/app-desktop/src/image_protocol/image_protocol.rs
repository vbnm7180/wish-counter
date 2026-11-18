//! Собственный протокол `wcimg` для картинок предметов.
//!
//! Интерфейс указывает картинку обычным тегом `<img src="...">` с адресом
//! вида `wcimg://localhost/genshin/hu_tao` (в Windows WebView2 тот же адрес
//! выглядит как `http://wcimg.localhost/genshin/hu_tao`). Запрос приходит
//! сюда, Rust достаёт картинку из кэша или скачивает её и отвечает байтами PNG.
//!
//! Так у интерфейса нет доступа ни к файлам, ни к сети: он может получить
//! только картинку известного предмета известной игры.

use std::sync::Arc;

use application::services::ImageService;
use domain::game::GameId;
use domain::wish::ItemId;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager, UriSchemeResponder};

use crate::state::AppState;

/// Имя протокола (схемы адреса).
pub const IMAGE_SCHEME: &str = "wcimg";
/// Тип содержимого ответа.
const PNG_CONTENT_TYPE: &str = "image/png";
/// Сколько секунд WebView может хранить картинку у себя, не спрашивая снова (сутки).
const BROWSER_CACHE_SECONDS: u32 = 86_400;

/// Обрабатывает запрос картинки. Вызывается Tauri для каждого адреса `wcimg://...`.
///
/// Получение картинки может потребовать сети, поэтому работа уходит в отдельный
/// поток (`spawn_blocking`), а ответ отправляется через `responder`, когда готов.
pub fn handle_request(app: &AppHandle, request: &Request<Vec<u8>>, responder: UriSchemeResponder) {
    let Some((game_id, item_id)) = parse_image_path(request.uri().path()) else {
        responder.respond(empty_response(StatusCode::BAD_REQUEST));
        return;
    };
    let service: Arc<ImageService> = Arc::clone(&app.state::<AppState>().image_service);
    tauri::async_runtime::spawn_blocking(move || {
        responder.respond(image_response(&service, &game_id, &item_id));
    });
}

/// Получает картинку и превращает результат в HTTP-ответ.
fn image_response(service: &ImageService, game_id: &GameId, item_id: &ItemId) -> Response<Vec<u8>> {
    match service.image(game_id, item_id) {
        Ok(Some(bytes)) => png_response(bytes),
        Ok(None) => empty_response(StatusCode::NOT_FOUND),
        Err(error) => {
            tracing::warn!(%error, "картинка не получена");
            empty_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Разбирает путь `/<игра>/<предмет>`. Обе части должны состоять только из
/// `a-z`, `0-9`, `_`, `-` — всё остальное отклоняется.
pub fn parse_image_path(path: &str) -> Option<(GameId, ItemId)> {
    let mut parts = path.trim_start_matches('/').split('/');
    let (game, item) = (parts.next()?, parts.next()?);
    let valid = parts.next().is_none() && is_safe_segment(game) && is_safe_segment(item);
    valid.then(|| (GameId::new(game), ItemId::new(item)))
}

/// Допустимая часть пути: не пустая, только `a-z`, `0-9`, `_`, `-`.
fn is_safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'))
}

/// Ответ с картинкой PNG.
fn png_response(bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, PNG_CONTENT_TYPE)
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
