//! Команды импорта и экспорта истории молитв.

use std::sync::Arc;

use application::services::{ExportedFile, ImportSummary};
use domain::game::GameId;
use domain::import::HistoryFormatId;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::background;
use crate::error::CommandError;
use crate::events;

/// Описание файлов JSON для фильтра в диалоге сохранения.
const JSON_FILTER_NAME: &str = "JSON";
/// Расширения файлов для фильтра в диалоге сохранения.
const JSON_EXTENSIONS: &[&str] = &["json"];

/// Результат экспорта для интерфейса.
#[derive(Debug, Serialize)]
pub struct ExportResult {
    /// Путь к сохранённому файлу; `None` — пользователь отменил сохранение.
    pub path: Option<String>,
    /// Сколько молитв в файле.
    pub wish_count: usize,
}
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
    format: HistoryFormatId,
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

/// Экспортирует историю молитв в файл: формирует содержимое, показывает
/// системный диалог «Сохранить как» и записывает файл туда, куда выбрал пользователь.
///
/// Вызов из интерфейса: `invoke('export_history', { gameId: 'genshin', format: 'paimon-moe' })`.
///
/// Диалог вызывается из Rust, поэтому интерфейсу не нужны права ни на диалоги,
/// ни на файловую систему. `blocking_save_file` ждёт выбора пользователя — поэтому
/// всё выполняется в отдельном потоке, а не в главном.
#[tauri::command]
pub async fn export_history(
    app: AppHandle,
    state: State<'_, AppState>,
    game_id: GameId,
    format: HistoryFormatId,
) -> Result<ExportResult, CommandError> {
    let service = Arc::clone(&state.export_service);
    let file =
        tauri::async_runtime::spawn_blocking(move || service.export(&game_id, &format)).await??;
    let wish_count = file.wish_count;
    let path =
        tauri::async_runtime::spawn_blocking(move || save_with_dialog(&app, &file)).await??;
    Ok(ExportResult { path, wish_count })
}

/// Спрашивает у пользователя, куда сохранить файл, и записывает его.
/// `Ok(None)` — пользователь закрыл диалог без сохранения.
fn save_with_dialog(app: &AppHandle, file: &ExportedFile) -> Result<Option<String>, CommandError> {
    let chosen = app
        .dialog()
        .file()
        .set_file_name(&file.file_name)
        .add_filter(JSON_FILTER_NAME, JSON_EXTENSIONS)
        .blocking_save_file();
    let Some(chosen) = chosen else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(CommandError::export_write)?;
    std::fs::write(&path, &file.content).map_err(CommandError::export_write)?;
    tracing::info!("файл экспорта сохранён");
    Ok(Some(path.display().to_string()))
}
