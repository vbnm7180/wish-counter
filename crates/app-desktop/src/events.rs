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

/// Данные события [`REFERENCE_UPDATED`]. В JSON: `{ "gameId": "genshin" }`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceUpdatedPayload<'a> {
    game_id: &'a GameId,
}

/// Сообщает интерфейсу, что справочник игры обновлён.
/// Ошибка отправки не критична: она пишется в лог.
pub fn emit_reference_updated(app: &AppHandle, game_id: &GameId) {
    if let Err(error) = app.emit(REFERENCE_UPDATED, ReferenceUpdatedPayload { game_id }) {
        tracing::warn!(%error, "не удалось отправить событие об обновлении справочника");
    }
}
