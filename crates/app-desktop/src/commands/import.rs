//! Команды импорта истории молитв.

use std::sync::Arc;

use application::services::ImportSummary;
use domain::game::GameId;
use domain::import::ImportFormatId;
use tauri::{AppHandle, State};

use crate::background;
use crate::error::CommandError;
use crate::events;
use crate::state::AppState;

/// Импортирует историю молитв из содержимого файла.
///
/// Вызов из интерфейса:
/// `invoke('import_history', { gameId: 'genshin', format: 'paimon-moe', content: '<текст файла>' })`.
///
/// Файл читает интерфейс (`<input type="file">`) и передаёт текст: так интерфейсу
/// не нужен доступ к файловой системе. Работа идёт в отдельном потоке, чтобы окно
/// не зависало. После успешного импорта:
/// - интерфейс получает событие `history-updated` и перечитывает статистику;
/// - в фоне докачиваются картинки всех предметов истории.
#[tauri::command]
pub async fn import_history(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: GameId,
    format: ImportFormatId,
    content: String,
) -> Result<ImportSummary, CommandError> {
    let service = Arc::clone(&state.import_service);
    let id = game_id.clone();
    let summary =
        tauri::async_runtime::spawn_blocking(move || service.import(&id, &format, &content))
            .await??;
    events::emit_history_updated(&app, &game_id);
    background::spawn_image_prefetch(
        Arc::clone(&state.image_service),
        game_id,
        summary.item_ids.clone(),
    );
    Ok(summary)
}
