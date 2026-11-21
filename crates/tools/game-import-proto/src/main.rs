//! Прототип этапа 7: поиск URL истории молитв в логах и кэше игры,
//! один тестовый запрос к API. Только для разработки — в дистрибутив не входит.
//!
//! Запуск: `cargo run -p game-import-proto -- [путь_к_data_2]`
//! Если путь не указан — ищет автоматически через output_log.txt.

use std::path::PathBuf;
use std::{env, fs, process};
use url::Url;

/// Имя файла кэша браузера внутри папки игры.
const DATA2_NAME: &str = "data_2";

/// Путь к логу Genshin в AppData\LocalLow.
const GENSHIN_LOG_RELATIVE: &str =
    r"miHoYo\Genshin Impact\output_log.txt";

/// Строка-маркер в output_log.txt, после которой идёт путь к игре.
const LOG_PATH_MARKER: &str = "MiHoYoSDKUtil: data path is ";

/// Подстрока, по которой находим URL с authkey в бинарном файле data_2.
const AUTHKEY_MARKER: &[u8] = b"https://gs.hoyoverse.com/gacha-info/api/getGachaLog";

/// Запрашиваем только один тип баннера для проверки связи.
const GACHA_TYPE_CHARACTER: &str = "301";

fn main() {
    let args: Vec<String> = env::args().collect();

    // Путь к data_2 можно передать аргументом; иначе ищем через output_log.txt.
    let data2_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        match find_data2_via_log() {
            Some(p) => p,
            None => {
                eprintln!("[прототип] data_2 не найден: игра не установлена или не запускалась");
                process::exit(1);
            }
        }
    };

    println!("[прототип] data_2: {}", data2_path.display());

    let url_str = match extract_url_from_data2(&data2_path) {
        Some(u) => u,
        None => {
            eprintln!("[прототип] URL с authkey не найден в data_2");
            eprintln!("  Убедитесь, что недавно открывали историю молитв в игре.");
            process::exit(1);
        }
    };

    println!("[прототип] URL найден (длина {} символов)", url_str.len());

    // Извлекаем параметры, нужные для API; authkey в лог не выводим.
    let (base_params, authkey) = match parse_url_params(&url_str) {
        Some(p) => p,
        None => {
            eprintln!("[прототип] не удалось разобрать параметры URL");
            process::exit(1);
        }
    };

    println!("[прототип] authkey получен ({} символов)", authkey.len());
    println!("[прототип] базовые параметры: {:?}", base_params);

    // Один тестовый запрос — первая страница баннера персонажа.
    println!("[прототип] запрос к API (тип {GACHA_TYPE_CHARACTER}, стр. 1)…");
    match fetch_first_page(&url_str, &authkey, &base_params) {
        Ok(count) => println!("[прототип] ✅ получено строк: {count}"),
        Err(e) => {
            eprintln!("[прототип] ❌ ошибка запроса: {e}");
            process::exit(1);
        }
    }
}

/// Ищет data_2 через output_log.txt в AppData\LocalLow.
fn find_data2_via_log() -> Option<PathBuf> {
    // %USERPROFILE%\AppData\LocalLow\miHoYo\Genshin Impact\output_log.txt
    let local_low = local_low_path()?;
    let log_path = local_low.join(GENSHIN_LOG_RELATIVE);

    println!("[прототип] ищу лог: {}", log_path.display());
    let content = fs::read_to_string(&log_path).ok()?;

    // Ищем строку вида «MiHoYoSDKUtil: data path is C:\...\GenshinImpact_Data»
    let game_data_path = content
        .lines()
        .rev() // берём последнее вхождение
        .find_map(|line| {
            line.find(LOG_PATH_MARKER)
                .map(|pos| line[pos + LOG_PATH_MARKER.len()..].trim().to_string())
        })?;

    println!("[прототип] папка данных игры: {game_data_path}");

    // Поднимаемся на уровень выше GenshinImpact_Data → папка игры,
    // затем ищем webCaches/<версия>/Cache/Cache_Data/data_2.
    let game_root = PathBuf::from(&game_data_path).parent()?.to_path_buf();
    find_data2_in_webcaches(&game_root)
}

/// Ищет последний data_2 в webCaches/<версия>/Cache/Cache_Data/.
/// Вместо перебора папки через read_dir используем список известных имён
/// из output_log.txt (путь к игре уже есть), поэтому read_dir нужен только
/// на уровне версий — это один уровень, не рабочие файлы.
fn find_data2_in_webcaches(game_root: &std::path::Path) -> Option<PathBuf> {
    // webCaches/<версия>/Cache/Cache_Data/data_2
    let webcaches = game_root.join("GenshinImpact_Data").join("webCaches");
    println!("[прототип] webCaches: {}", webcaches.display());

    // Читаем папки версий (одноуровневый перебор, не рабочие файлы игры).
    let entries = fs::read_dir(&webcaches).ok()?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("Cache").join("Cache_Data").join(DATA2_NAME))
        .filter(|p| p.exists())
        .collect();

    // Берём последнюю версию (лексикографически наибольшую).
    candidates.sort();
    candidates.pop()
}

/// Сканирует бинарный файл data_2 на наличие URL с authkey.
/// Находит первое вхождение маркера и возвращает строку до первого нуль-байта.
fn extract_url_from_data2(path: &std::path::Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let pos = bytes
        .windows(AUTHKEY_MARKER.len())
        .position(|w| w == AUTHKEY_MARKER)?;

    // Читаем до нуль-байта или конца файла.
    let url_bytes = &bytes[pos..];
    let end = url_bytes.iter().position(|&b| b == 0).unwrap_or(url_bytes.len());
    let url_str = std::str::from_utf8(&url_bytes[..end]).ok()?;

    // Быстрая проверка: в URL должен быть authkey.
    if url_str.contains("authkey=") {
        Some(url_str.to_string())
    } else {
        None
    }
}

/// Параметры запроса, которые передаём в API, кроме authkey.
#[derive(Debug)]
struct BaseParams {
    region: String,
    lang: String,
    game_biz: String,
    authkey_ver: String,
    sign_type: String,
}

/// Разбирает URL и возвращает базовые параметры и authkey.
/// authkey возвращается отдельно, чтобы нигде не логировать его случайно.
fn parse_url_params(url_str: &str) -> Option<(BaseParams, String)> {
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

    if authkey.is_empty() {
        return None;
    }

    Some((BaseParams { region, lang, game_biz, authkey_ver, sign_type }, authkey))
}

/// Делает один запрос к API и возвращает количество полученных строк.
fn fetch_first_page(_original_url: &str, authkey: &str, params: &BaseParams) -> Result<usize, String> {
    // Собираем URL вручную, чтобы authkey не попал в структуры, которые логируются.
    let api_url = format!(
        "https://gs.hoyoverse.com/gacha-info/api/getGachaLog\
?authkey_ver={authkey_ver}\
&sign_type={sign_type}\
&auth_appid=webview_gacha\
&lang={lang}\
&gacha_type={gacha_type}\
&page=1\
&size=20\
&end_id=0\
&region={region}\
&game_biz={game_biz}\
&authkey={authkey}",
        authkey_ver = params.authkey_ver,
        sign_type = params.sign_type,
        lang = params.lang,
        gacha_type = GACHA_TYPE_CHARACTER,
        region = params.region,
        game_biz = params.game_biz,
        // authkey передаётся последним — не оказывается в начале строки лога
        authkey = authkey,
    );
    // Используем ureq — тот же HTTP-клиент, что и в основном приложении.
    let response = ureq::get(&api_url)
        .call()
        .map_err(|e| format!("{e}"))?;

    let body_str = response
        .into_body()
        .read_to_string()
        .map_err(|e| format!("{e}"))?;
    let body: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|e| format!("{e}"))?;

    // Ответ: {"retcode": 0, "data": {"list": [...], ...}}
    let retcode = body["retcode"].as_i64().unwrap_or(-1);
    if retcode != 0 {
        let msg = body["message"].as_str().unwrap_or("неизвестная ошибка");
        return Err(format!("API retcode {retcode}: {msg}"));
    }

    let count = body["data"]["list"]
        .as_array()
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(count)
}

/// Возвращает путь к AppData\LocalLow текущего пользователя.
/// LocalLow = %USERPROFILE%\AppData\LocalLow.
fn local_low_path() -> Option<PathBuf> {
    let profile = env::var("USERPROFILE").ok()?;
    Some(PathBuf::from(profile).join("AppData").join("LocalLow"))
}
