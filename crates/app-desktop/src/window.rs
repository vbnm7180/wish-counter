//! Создание главного окна приложения.
//!
//! Окно описано в `tauri.conf.json` с флагом `"create": false`: Tauri не
//! создаёт его сам, а мы создаём его здесь. Это нужно, чтобы указать
//! WebView2 папку для служебных данных внутри `data/` — иначе он пишет
//! в профиль пользователя (`%LOCALAPPDATA%`), что недопустимо для
//! portable-версии.

use std::error::Error;
use std::path::PathBuf;

use tauri::{App, WebviewWindowBuilder};

/// Метка (label) главного окна из `tauri.conf.json`.
const MAIN_WINDOW_LABEL: &str = "main";

/// Создаёт главное окно по описанию из конфигурации и указывает
/// WebView2 папку служебных данных `webview_dir`.
///
/// Тип ошибки `Box<dyn Error>` требует `tauri::Builder::setup`:
/// это «любая ошибка», упакованная в кучу.
pub fn create_main_window(app: &mut App, webview_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == MAIN_WINDOW_LABEL)
        .ok_or("в tauri.conf.json нет описания главного окна")?
        .clone();
    WebviewWindowBuilder::from_config(app.handle(), &window_config)?
        .data_directory(webview_dir)
        .build()?;
    Ok(())
}
