//! Команды фонов приложения: список, добавление своей картинки, удаление.
//!
//! Выбор фона сохраняется обычной командой `update_settings` (поле `background`).

use std::path::Path;
use std::sync::Arc;

use application::services::{BackgroundService, MAX_BACKGROUND_BYTES};
use domain::background::{BackgroundId, BackgroundImage};
use domain::settings::AppSettings;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::CommandError;
use crate::state::AppState;

/// Описание картинок для фильтра в диалоге выбора файла.
const IMAGE_FILTER_NAME: &str = "PNG, JPEG, WebP";
/// Расширения картинок для фильтра в диалоге (сам формат проверяется по содержимому).
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

/// Возвращает добавленные картинки фона (старые первыми).
///
/// Вызов из интерфейса: `invoke('list_backgrounds')`.
#[tauri::command]
pub fn list_backgrounds(state: State<'_, AppState>) -> Result<Vec<BackgroundImage>, CommandError> {
    Ok(state.background_service.list()?)
}

/// Показывает диалог выбора файла и добавляет выбранную картинку в фоны.
/// `None` — пользователь закрыл диалог.
///
/// Вызов из интерфейса: `invoke('add_background')`.
///
/// Файл выбирает и читает Rust: интерфейсу не нужны права ни на диалоги,
/// ни на файловую систему. Диалог ждёт пользователя, поэтому всё — в отдельном потоке.
#[tauri::command]
pub async fn add_background(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<BackgroundImage>, CommandError> {
    let service = Arc::clone(&state.background_service);
    tauri::async_runtime::spawn_blocking(move || pick_and_add(&app, &service)).await?
}

/// Удаляет картинку фона; если она была выбрана, фон сбрасывается.
/// Возвращает настройки после удаления.
///
/// Вызов из интерфейса: `invoke('remove_background', { id })`.
#[tauri::command]
pub fn remove_background(
    state: State<'_, AppState>,
    id: BackgroundId,
) -> Result<AppSettings, CommandError> {
    Ok(state.background_service.remove(&id)?)
}

/// Спрашивает файл у пользователя, читает его и передаёт сервису.
fn pick_and_add(
    app: &AppHandle,
    service: &BackgroundService,
) -> Result<Option<BackgroundImage>, CommandError> {
    let chosen = app
        .dialog()
        .file()
        .add_filter(IMAGE_FILTER_NAME, IMAGE_EXTENSIONS)
        .blocking_pick_file();
    let Some(chosen) = chosen else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(CommandError::background_read)?;
    let bytes = read_limited(&path)?;
    Ok(Some(service.add(&bytes)?))
}

/// Читает файл, если он не больше допустимого размера. Размер проверяется до чтения,
/// чтобы не загружать в память огромный файл (путь в лог не пишется — в нём имя пользователя).
fn read_limited(path: &Path) -> Result<Vec<u8>, CommandError> {
    let size = std::fs::metadata(path)
        .map_err(CommandError::background_read)?
        .len();
    if size > MAX_BACKGROUND_BYTES {
        return Err(CommandError::background_too_large());
    }
    std::fs::read(path).map_err(CommandError::background_read)
}
