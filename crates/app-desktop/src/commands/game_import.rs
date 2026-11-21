//! Команды импорта истории молитв напрямую из игры и очистки данных.

use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};

use application::services::{AuthKeyParams, ClearSummary, GameApiImportSummary};
use domain::game::GameId;
use tauri::State;
use url::Url;

use crate::background;
use crate::error::CommandError;
use crate::events;
use crate::state::AppState;

/// Подстрока, по которой находим URL с authkey в бинарном файле data_2.
/// Оба хоста: старый (gs.hoyoverse.com) и новый (public-operation-hk4e-sg.hoyoverse.com).
const AUTHKEY_MARKER: &[u8] = b"getGachaLog";

/// Начало URL — от него отсчитываем назад от маркера до начала адреса.
const HTTPS_PREFIX: &[u8] = b"https://";

/// Домены серверов HoYoverse (глобальные) и miHoYo (Китай). authkey отправляется
/// только на них — даже если в кэше окажется ссылка на чужой сайт.
const TRUSTED_API_DOMAINS: &[&str] = &["hoyoverse.com", "mihoyo.com"];

/// `game_biz` для глобальных серверов: в ссылках новых версий игры его нет.
const DEFAULT_GAME_BIZ: &str = "hk4e_global";

/// Проверяет, что хост — сервер HoYoverse/miHoYo (сам домен или его поддомен).
fn is_trusted_api_host(host: &str) -> bool {
    TRUSTED_API_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

/// Путь к логу Genshin в AppData\LocalLow относительно %USERPROFILE%.
const GENSHIN_LOG_RELATIVE: &str = r"AppData\LocalLow\miHoYo\Genshin Impact\output_log.txt";

/// Строка-маркер в output_log.txt, после которой идёт путь к папке данных игры.
/// Используется для старых версий (до HoYoPlay).
const LOG_PATH_MARKER: &str = "MiHoYoSDKUtil: data path is ";

/// Подстрока, по которой определяем папку данных в любой версии игры (включая HoYoPlay).
const GENSHIN_DATA_DIR: &str = "GenshinImpact_Data";

/// Имя файла кэша браузера внутри папки Cache_Data.
const DATA2_NAME: &str = "data_2";

/// Запускает импорт истории молитв из API игры.
///
/// Автоматически ищет data_2 через output_log.txt.
/// Возвращает краткий итог; authkey нигде не логируется и не сохраняется.
///
/// Вызов из интерфейса: `invoke('import_from_game', { gameId: 'genshin' })`.
#[tauri::command]
pub async fn import_from_game(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<GameApiImportSummary, CommandError> {
    let service = Arc::clone(&state.game_api_import_service);
    let image_service = Arc::clone(&state.image_service);
    let id = game_id.clone();

    let summary = tauri::async_runtime::spawn_blocking(move || {
        let params = find_authkey_params().map_err(|e| CommandError::game_import(&e))?;
        service.import(&id, &params).map_err(CommandError::from)
    })
    .await??;

    events::emit_history_updated(&app, &game_id);
    background::spawn_image_prefetch(image_service, game_id, summary.item_ids.clone());
    Ok(summary)
}

/// Удаляет все молитвы и профиль игры (история + сервер + UID).
/// Используется при смене аккаунта.
///
/// Вызов из интерфейса: `invoke('clear_game_data', { gameId: 'genshin' })`.
#[tauri::command]
pub async fn clear_game_data(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    game_id: GameId,
) -> Result<ClearSummary, CommandError> {
    let service = Arc::clone(&state.clear_game_service);
    let id = game_id.clone();
    let summary =
        tauri::async_runtime::spawn_blocking(move || service.clear(&id).map_err(CommandError::from))
            .await??;
    events::emit_history_updated(&app, &game_id);
    Ok(summary)
}

/// Ищет URL с authkey в веб-кэше игры через output_log.txt.
fn find_authkey_params() -> Result<AuthKeyParams, String> {
    let profile = env::var("USERPROFILE")
        .map_err(|_| "переменная USERPROFILE не задана".to_string())?;
    let log_path = PathBuf::from(&profile).join(GENSHIN_LOG_RELATIVE);

    tracing::debug!(path = %log_path.display(), "ищу output_log.txt");
    let content = fs::read_to_string(&log_path)
        .map_err(|e| format!("не удалось прочитать output_log.txt ({}): {e}", log_path.display()))?;

    let game_root = game_root_from_log(&content)
        .ok_or_else(|| "папка игры не найдена в output_log.txt — возможно, другая версия игры".to_string())?;

    let webcaches = game_root.join("GenshinImpact_Data").join("webCaches");
    tracing::debug!(path = %webcaches.display(), "ищу webCaches");

    let data2_path = find_data2_in_webcaches(&game_root)
        .ok_or_else(|| format!("data_2 не найден в {} — откройте историю молитв в игре", webcaches.display()))?;

    tracing::debug!(path = %data2_path.display(), "нашёл data_2");

    let url_str = extract_url_from_data2(&data2_path)
        .ok_or_else(|| format!("URL с authkey не найден в {} — откройте историю молитв в игре и попробуйте снова", data2_path.display()))?;

    parse_authkey_params(&url_str)
        .ok_or_else(|| "не удалось разобрать параметры authkey из URL".to_string())
}

/// Извлекает корневую папку игры из содержимого output_log.txt.
///
/// Поддерживает две версии:
/// - старую: строка «MiHoYoSDKUtil: data path is <путь>\GenshinImpact_Data»
/// - новую (HoYoPlay): любая строка, содержащая «GenshinImpact_Data»;
///   путь берётся от буквы диска (напр. «C:\») до этой папки.
///
/// Возвращает папку, содержащую `GenshinImpact_Data` (корень установки игры).
fn game_root_from_log(content: &str) -> Option<PathBuf> {
    // Сначала ищем точный маркер старой версии.
    if let Some(game_root) = content.lines().rev().find_map(|line| {
        line.find(LOG_PATH_MARKER).and_then(|pos| {
            let data_path = line[pos + LOG_PATH_MARKER.len()..].trim();
            PathBuf::from(data_path).parent().map(|p| p.to_path_buf())
        })
    }) {
        return Some(game_root);
    }

    // Иначе ищем последнюю строку с «GenshinImpact_Data» и берём путь
    // от буквы диска (напр. «C:\») до этой папки.
    // Пример строки: «TelemetryInterface path:C:\Program Files\HoYoPlay\games\Genshin Impact game\GenshinImpact_Data\SDKCaches»
    content.lines().rev().find_map(|line| {
        let pos = line.find(GENSHIN_DATA_DIR)?;
        let before = &line[..pos];
        // Ищем последнее вхождение паттерна «X:\» или «X:/» (буква диска).
        let bytes = before.as_bytes();
        let drive_start = (0..bytes.len().saturating_sub(2)).rev().find(|&i| {
            bytes[i].is_ascii_alphabetic()
                && bytes.get(i + 1) == Some(&b':')
                && matches!(bytes.get(i + 2), Some(b'\\') | Some(b'/'))
        })?;
        let root = PathBuf::from(&before[drive_start..]);
        if root.exists() { Some(root) } else { None }
    })
}

/// Ищет последний data_2 в webCaches/<версия>/Cache/Cache_Data/.
/// Перебирает только одноуровневые папки версий — рабочие файлы игры не затрагиваются.
fn find_data2_in_webcaches(game_root: &std::path::Path) -> Option<PathBuf> {
    let webcaches = game_root
        .join("GenshinImpact_Data")
        .join("webCaches");

    // Одноуровневый перебор папок версий — не рабочие файлы игры.
    let entries = fs::read_dir(&webcaches).ok()?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| {
            e.path()
                .join("Cache")
                .join("Cache_Data")
                .join(DATA2_NAME)
        })
        .filter(|p| p.exists())
        .collect();

    candidates.sort();
    candidates.pop()
}

/// Сканирует data_2 и извлекает URL с authkey.
/// Маркер — `getGachaLog`; от него идём назад до `https://` и вперёд до нуль-байта.
fn extract_url_from_data2(path: &std::path::Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    // `rposition` — последнее вхождение: в кэше может быть несколько ссылок, последняя самая свежая.
    let marker_pos = bytes
        .windows(AUTHKEY_MARKER.len())
        .rposition(|w| w == AUTHKEY_MARKER)?;

    // Идём назад от маркера до начала «https://».
    // Перед адресом в кэше бывает служебный префикс вроде «1/0/» — он отбрасывается.
    let url_start = bytes[..marker_pos]
        .windows(HTTPS_PREFIX.len())
        .rposition(|w| w == HTTPS_PREFIX)?;

    // Читаем до нуль-байта или конца файла.
    let url_bytes = &bytes[url_start..];
    let end = url_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(url_bytes.len());
    let url_str = std::str::from_utf8(&url_bytes[..end]).ok()?;

    if url_str.contains("authkey=") {
        Some(url_str.to_string())
    } else {
        None
    }
}

/// Разбирает URL и возвращает параметры для API.
/// authkey извлекается отдельно и не попадает в структуры, которые логируются.
fn parse_authkey_params(url_str: &str) -> Option<AuthKeyParams> {
    let url = Url::parse(url_str).ok()?;
    let mut region = String::new();
    let mut lang = String::new();
    let mut game_biz = String::new();
    let mut authkey_ver = String::new();
    let mut sign_type = String::new();
    let mut authkey = String::new();

    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "region" => region = v.into_owned(),
            "lang" => lang = v.into_owned(),
            "game_biz" => game_biz = v.into_owned(),
            "authkey_ver" => authkey_ver = v.into_owned(),
            "sign_type" => sign_type = v.into_owned(),
            "authkey" => authkey = v.into_owned(),
            _ => {}
        }
    }

    if authkey.is_empty() || !is_trusted_api_host(url.host_str()?) {
        return None;
    }
    if game_biz.is_empty() {
        game_biz = DEFAULT_GAME_BIZ.to_string();
    }

    // Адрес API без параметров: схема + хост + путь из найденной ссылки.
    let api_url = format!("https://{}{}", url.host_str()?, url.path());

    Some(AuthKeyParams {
        api_url,
        authkey,
        region,
        lang,
        game_biz,
        authkey_ver,
        sign_type,
    })
}
