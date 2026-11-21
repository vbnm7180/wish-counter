//! Импорт истории молитв напрямую из API игры.
//!
//! Схема:
//! 1. Интерфейс передаёт URL с `authkey` (найденный Rust-ом в веб-кэше игры).
//! 2. Сервис по очереди запрашивает каждый баннер страницами по 20 молитв.
//! 3. Молитвы сохраняются в хранилище — дубликаты пропускаются, поэтому
//!    повторный импорт безопасен.
//!
//! `authkey` живёт только в памяти и никуда не пишется и не логируется.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use domain::game::GameId;
use domain::import::GameApiError;
use domain::reference::ReferenceData;
use domain::region::ServerRegion;
use domain::wish::ItemId;
use serde::Serialize;
use thiserror::Error;

use super::{GameCatalog, ReferenceService};
use crate::ports::{GameApiClient, GameApiClientError, ProfileRepository, RepositoryError, WishRepository};
use crate::services::reference_service::ReferenceError;

/// Сколько молитв запрашивать за одну страницу (API допускает до 20).
const PAGE_SIZE: u32 = 20;

/// Язык ответа API. Всегда английский: предметы ищутся в справочнике по английскому имени,
/// а с языком из ссылки игры (например, `ru`) пришли бы русские названия.
const API_LANG: &str = "en-us";

/// Пауза между запросами страниц, чтобы не нагружать сервер игры.
const REQUEST_DELAY: Duration = Duration::from_millis(250);

/// Предел страниц на один баннер: 20 × 10 000 = 200 000 молитв.
/// Защищает от бесконечного цикла, если API всегда возвращает `last_id`.
const MAX_PAGES_PER_BANNER: usize = 10_000;

/// Параметры запроса из URL, найденного в веб-кэше игры.
/// `authkey` хранится отдельно от публичных полей.
#[derive(Debug, Clone)]
pub struct AuthKeyParams {
    /// Адрес API без параметров (`https://<хост>/gacha_info/api/getGachaLog`).
    /// Берётся из ссылки в кэше игры: у разных серверов и версий игры хост разный.
    pub api_url: String,
    /// Ключ доступа к API истории (секрет: не логировать и не сохранять).
    pub authkey: String,
    /// Регион (`eu`, `asia`, …).
    pub region: String,
    /// Язык ответа.
    pub lang: String,
    /// Идентификатор игры для сервера (`hk4e_eu` и т.п.).
    pub game_biz: String,
    /// Версия ключа.
    pub authkey_ver: String,
    /// Тип подписи.
    pub sign_type: String,
}

/// Ошибка импорта из API игры.
#[derive(Debug, Error)]
pub enum GameApiImportError {
    /// Игра не зарегистрирована.
    #[error("игра «{0}» не поддерживается")]
    UnknownGame(GameId),
    /// Игра не поддерживает импорт из API.
    #[error("игра «{0}» не поддерживает импорт из API")]
    NoApiParser(GameId),
    /// Ошибка сети.
    #[error(transparent)]
    Network(#[from] GameApiClientError),
    /// Ошибка разбора ответа или ошибка API.
    #[error(transparent)]
    Api(#[from] GameApiError),
    /// Ошибка хранилища.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    /// Ошибка справочника.
    #[error(transparent)]
    Reference(#[from] ReferenceError),
}

/// Итог импорта из API для показа пользователю.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameApiImportSummary {
    /// Сколько молитв прочитано из API.
    pub total_in_api: usize,
    /// Сколько молитв добавлено (новых).
    pub added: usize,
    /// Сколько молитв уже было сохранено раньше.
    pub already_present: usize,
    /// Сервер игрока из ссылки игры (`None` — код региона неизвестен).
    pub server_region: Option<ServerRegion>,
    /// Предметы из истории — для докачки их картинок.
    #[serde(skip)]
    pub item_ids: Vec<ItemId>,
}

/// Импортирует историю молитв напрямую из API игры.
pub struct GameApiImportService {
    catalog: Arc<GameCatalog>,
    references: Arc<ReferenceService>,
    wish_repository: Arc<dyn WishRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
    client: Arc<dyn GameApiClient>,
}

impl GameApiImportService {
    /// Создаёт сервис. `profile_repository` — для сохранения сервера игрока,
    /// `client` — HTTP-клиент для запросов к серверу игры.
    pub fn new(
        catalog: Arc<GameCatalog>,
        references: Arc<ReferenceService>,
        wish_repository: Arc<dyn WishRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
        client: Arc<dyn GameApiClient>,
    ) -> Self {
        Self {
            catalog,
            references,
            wish_repository,
            profile_repository,
            client,
        }
    }

    /// Импортирует всю историю молитв игры через API.
    ///
    /// `params` — параметры с `authkey`; сервис собирает адреса запросов сам.
    pub fn import(
        &self,
        game_id: &GameId,
        params: &AuthKeyParams,
    ) -> Result<GameApiImportSummary, GameApiImportError> {
        let game = self
            .catalog
            .find(game_id)
            .ok_or_else(|| GameApiImportError::UnknownGame(game_id.clone()))?;
        let parser = game
            .game_api_parser()
            .ok_or_else(|| GameApiImportError::NoApiParser(game_id.clone()))?;
        let reference = self.references.reference(game_id)?;

        let mut all_wishes = Vec::new();

        for (gacha_type, _name) in &parser.gacha_type_codes() {
            let wishes = self.fetch_banner(&parser, &reference, params, gacha_type)?;
            all_wishes.extend(wishes);
        }

        let total_in_api = all_wishes.len();
        let item_ids = collect_item_ids(&all_wishes);
        let added = self.wish_repository.add_wishes(game_id, &all_wishes)?;

        // Без сервера нельзя определить исход 50/50: время баннеров зависит от часового пояса.
        // Сервер берётся из ссылки игры — он точно соответствует аккаунту, с которого идёт импорт.
        let server_region = parser.server_region(&params.region);
        if let Some(region) = server_region {
            self.profile_repository.set_server_region(game_id, region)?;
        }

        Ok(GameApiImportSummary {
            total_in_api,
            added,
            already_present: total_in_api - added,
            server_region,
            item_ids,
        })
    }

    /// Запрашивает все страницы одного баннера и возвращает молитвы
    /// в хронологическом порядке (от старых к новым).
    fn fetch_banner(
        &self,
        parser: &Arc<dyn domain::import::GameApiParser>,
        reference: &ReferenceData,
        params: &AuthKeyParams,
        gacha_type: &str,
    ) -> Result<Vec<domain::wish::Wish>, GameApiImportError> {
        let mut pages: Vec<domain::import::ApiWishPage> = Vec::new();
        let mut last_id: Option<String> = None;

        for _ in 0..MAX_PAGES_PER_BANNER {
            let url = build_page_url(params, gacha_type, last_id.as_deref());
            let body = self.client.get(&url)?;
            let page = parser.parse_page(&body, reference)?;

            let new_last_id = page.last_id.clone();
            pages.push(page);

            // Пустой список или отсутствие нового last_id — последняя страница.
            let Some(new_last_id) = new_last_id else { break };
            if Some(&new_last_id) == last_id.as_ref() {
                break;
            }
            last_id = Some(new_last_id);

            // Пауза, чтобы не нагружать сервер игры.
            thread::sleep(REQUEST_DELAY);
        }

        // Страницы идут от новых к старым; внутри страницы тоже новые сверху.
        // Собираем в обратном порядке, чтобы получить хронологию.
        let mut wishes: Vec<domain::wish::Wish> = pages
            .into_iter()
            .rev()
            .flat_map(|page| page.wishes)
            .collect();

        // Внутри одной страницы тоже был reverse в парсере, поэтому после
        // внешнего reverse порядок хронологический — но для устойчивости сортируем
        // по времени и id (ординальный порядок для одинаковых времён).
        wishes.sort_by(|a, b| a.time.as_str().cmp(b.time.as_str()).then(a.id.as_str().cmp(b.id.as_str())));

        Ok(wishes)
    }
}

/// Собирает уникальные идентификаторы предметов из молитв.
fn collect_item_ids(wishes: &[domain::wish::Wish]) -> Vec<ItemId> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for wish in wishes {
        if seen.insert(wish.item_id.clone()) {
            result.push(wish.item_id.clone());
        }
    }
    result
}

/// Собирает адрес запроса одной страницы API.
/// `authkey` передаётся последним параметром, чтобы не оказываться в начале строки.
fn build_page_url(params: &AuthKeyParams, gacha_type: &str, last_id: Option<&str>) -> String {
    // Все значения кодируются: в authkey есть «+», «/», «=», которые иначе испортят адрес.
    format!(
        "{api_url}\
?authkey_ver={authkey_ver}\
&sign_type={sign_type}\
&auth_appid=webview_gacha\
&lang={lang}\
&gacha_type={gacha_type}\
&page=1\
&size={page_size}\
&end_id={end_id}\
&region={region}\
&game_biz={game_biz}\
&authkey={authkey}",
        api_url = params.api_url,
        authkey_ver = percent_encode(&params.authkey_ver),
        sign_type = percent_encode(&params.sign_type),
        lang = API_LANG,
        gacha_type = percent_encode(gacha_type),
        page_size = PAGE_SIZE,
        end_id = percent_encode(last_id.unwrap_or("0")),
        region = percent_encode(&params.region),
        game_biz = percent_encode(&params.game_biz),
        authkey = percent_encode(&params.authkey),
    )
}

/// Кодирует значение параметра адреса (RFC 3986): буквы, цифры и `-._~` остаются,
/// остальные байты превращаются в `%XX`.
fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            // `{:02X}` — две шестнадцатеричные цифры в верхнем регистре.
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
