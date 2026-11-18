//! Команды справочных данных.

use std::sync::Arc;

use application::services::{ReferenceStatus, UpdateMode, UpdateOutcome};
use domain::game::GameId;
use tauri::{AppHandle, State};

use crate::error::CommandError;
use crate::events;
use crate::state::AppState;

/// Возвращает сводку о справочнике игры (число персонажей, оружия, баннеров,
/// последний баннер, версия данных) и время последней проверки обновлений.
///
/// Вызов из интерфейса: `invoke('get_reference_summary', { gameId: 'genshin' })`.
#[tauri::command]
pub fn get_reference_summary(
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<ReferenceStatus, CommandError> {
    Ok(state.reference_updater.status(&game_id)?)
}

/// Проверяет обновление справочника прямо сейчас (кнопка «Обновить данные»).
///
/// Вызов из интерфейса: `invoke('update_reference', { gameId: 'genshin' })`.
///
/// Команда `async`: обычные (синхронные) команды Tauri выполняются в главном
/// потоке, и сетевой запрос «заморозил» бы окно. Здесь работа уходит в
/// отдельный поток через `spawn_blocking`, а окно остаётся отзывчивым.
#[tauri::command]
pub async fn update_reference(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<UpdateOutcome, CommandError> {
    let updater = Arc::clone(&state.reference_updater);
    let id = game_id.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        updater.update(&id, UpdateMode::Force, chrono::Utc::now().naive_utc())
    })
    .await??;
    if matches!(outcome, UpdateOutcome::Updated { .. }) {
        events::emit_reference_updated(&app, &game_id);
    }
    Ok(outcome)
}
