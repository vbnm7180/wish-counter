//! События, которые ядро отправляет интерфейсу.
//!
//! Команды — это «интерфейс спрашивает, ядро отвечает». События — наоборот:
//! ядро само сообщает интерфейсу, что что-то изменилось (например,
//! справочник обновился в фоне).

use domain::game::GameId;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Имя события «справочник игры обновлён». Интерфейс подписывается на него
/// в `ui/src/lib/api/events.ts`.
pub const REFERENCE_UPDATED: &str = "reference-updated";

/// Имя события «история молитв игры изменилась» (например, после импорта).
pub const HISTORY_UPDATED: &str = "history-updated";

/// Данные событий об игре. В JSON: `{ "gameId": "genshin" }`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GamePayload<'a> {
    game_id: &'a GameId,
}

/// Сообщает интерфейсу, что справочник игры обновлён.
/// Ошибка отправки не критична: она пишется в лог.
pub fn emit_reference_updated(app: &AppHandle, game_id: &GameId) {
    emit_game_event(app, REFERENCE_UPDATED, game_id);
}

/// Сообщает интерфейсу, что история молитв игры изменилась.
pub fn emit_history_updated(app: &AppHandle, game_id: &GameId) {
    emit_game_event(app, HISTORY_UPDATED, game_id);
}

/// Отправляет событие с идентификатором игры. Ошибка отправки не критична: пишется в лог.
fn emit_game_event(app: &AppHandle, event: &str, game_id: &GameId) {
    if let Err(error) = app.emit(event, GamePayload { game_id }) {
        tracing::warn!(%error, event, "не удалось отправить событие интерфейсу");
    }
}
