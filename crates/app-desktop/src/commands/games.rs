//! Команды, связанные с играми: список игр и статистика молитв.

use application::services::GameStatistics;
use domain::game::{GameId, GameInfo};
use domain::profile::{GameProfile, PlayerUid};
use domain::region::ServerRegion;
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

/// Возвращает список поддерживаемых игр.
///
/// Вызов из интерфейса: `invoke('list_games')`.
#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> Vec<GameInfo> {
    state.game_catalog.list_games()
}

/// Возвращает статистику молитв игры: итоги и карточки баннеров.
///
/// Вызов из интерфейса: `invoke('get_game_statistics', { gameId: 'genshin' })`.
/// Tauri переводит имя параметра `game_id` в `gameId` для JavaScript.
#[tauri::command]
pub fn get_game_statistics(
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<GameStatistics, CommandError> {
    Ok(state.statistics_service.game_statistics(&game_id)?)
}

/// Сохраняет сервер игрока для игры.
///
/// Вызов из интерфейса: `invoke('set_server_region', { gameId: 'genshin', region: 'europe' })`.
/// Недопустимое значение `region` отклоняется ещё при разборе JSON (enum `ServerRegion`).
#[tauri::command]
pub fn set_server_region(
    state: State<'_, AppState>,
    game_id: GameId,
    region: ServerRegion,
) -> Result<(), CommandError> {
    Ok(state.profile_service.set_server_region(&game_id, region)?)
}

/// Возвращает профиль игрока: сервер и UID (`null` — не заданы).
///
/// Вызов из интерфейса: `invoke('get_game_profile', { gameId: 'genshin' })`.
#[tauri::command]
pub fn get_game_profile(
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<GameProfile, CommandError> {
    Ok(state.profile_service.profile(&game_id)?)
}

/// Сохраняет UID игрока. Строка проверяется: только цифры, 6–12 знаков.
///
/// Вызов из интерфейса: `invoke('set_player_uid', { gameId: 'genshin', uid: '700000000' })`.
#[tauri::command]
pub fn set_player_uid(
    state: State<'_, AppState>,
    game_id: GameId,
    uid: String,
) -> Result<(), CommandError> {
    let uid = PlayerUid::parse(&uid)?;
    Ok(state.profile_service.set_uid(&game_id, &uid)?)
}
