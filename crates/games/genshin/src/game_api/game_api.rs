//! Разбор ответа API истории молитв Genshin Impact.
//!
//! API возвращает JSON-страницы. Этот модуль преобразует их в доменные объекты
//! [`Wish`]. Сетевые запросы делает `application::services::GameApiImportService`,
//! а не этот модуль — здесь только правила преобразования данных.

use domain::import::{ApiWishPage, GameApiError, GameApiParser, GachaTypeCodes};
use domain::reference::ReferenceData;
use domain::region::ServerRegion;
use domain::wish::{BannerCode, ItemKind, Rarity, Wish, WishId, WishTime};
use serde::Deserialize;

use crate::banner_codes;

/// Разборщик ответов API истории молитв Genshin Impact.
///
/// Реализует [`GameApiParser`]: знает коды баннеров и умеет разбирать JSON.
pub struct GenshinApiParser;

impl GameApiParser for GenshinApiParser {
    /// Коды всех баннеров Genshin, по которым нужно запрашивать историю.
    fn gacha_type_codes(&self) -> GachaTypeCodes {
        vec![
            (banner_codes::CHARACTER.value().to_string(),   "Баннер события персонажа".into()),
            (banner_codes::CHARACTER_2.value().to_string(), "Баннер события персонажа 2".into()),
            (banner_codes::WEAPON.value().to_string(),      "Баннер оружия".into()),
            (banner_codes::CHRONICLED.value().to_string(),  "Хроники странствий".into()),
            (banner_codes::STANDARD.value().to_string(),    "Стандартный баннер".into()),
            (banner_codes::BEGINNER.value().to_string(),    "Баннер новичка".into()),
        ]
    }

    /// Сервер по коду из ссылки игры: `os_usa`, `os_euro`, `os_asia`, `os_cht` (TW/HK/MO — тоже UTC+8),
    /// китайские `cn_gf01` (официальный) и `cn_qd01` (Bilibili).
    fn server_region(&self, region_code: &str) -> Option<ServerRegion> {
        match region_code {
            "os_usa" => Some(ServerRegion::America),
            "os_euro" => Some(ServerRegion::Europe),
            "os_asia" | "os_cht" => Some(ServerRegion::Asia),
            "cn_gf01" | "cn_qd01" => Some(ServerRegion::China),
            _ => None,
        }
    }

    /// Разбирает одну страницу ответа API и возвращает молитвы + id последней записи.
    fn parse_page(&self, json: &str, reference: &ReferenceData) -> Result<ApiWishPage, GameApiError> {
        let data = ApiResponseData::parse(json)?;
        let mut wishes = Vec::with_capacity(data.list.len());
        let mut unknown_items = Vec::new();

        for record in &data.list {
            if let Some(wish) = convert_record(record, reference, &mut unknown_items)
                .map_err(|e| GameApiError::InvalidResponse(e.to_string()))?
            {
                wishes.push(wish);
            }
        }

        if !unknown_items.is_empty() {
            return Err(GameApiError::UnknownItems(unknown_items));
        }

        let last_id = data.list.last().map(|r| r.id.clone());

        // API возвращает молитвы от новых к старым; переворачиваем,
        // чтобы хранилище получало их в хронологическом порядке.
        wishes.reverse();

        Ok(ApiWishPage { wishes, last_id })
    }
}

/// Одна запись из списка `data.list` ответа API.
#[derive(Debug, Deserialize)]
struct ApiWishRecord {
    id: String,
    gacha_type: String,
    name: String,
    item_type: String,
    rank_type: String,
    time: String,
}

/// Тело успешного ответа API (поле `data`).
#[derive(Debug, Deserialize)]
struct ApiResponseData {
    list: Vec<ApiWishRecord>,
}

/// Корневой ответ API истории молитв.
#[derive(Debug, Deserialize)]
struct ApiRoot {
    retcode: i64,
    message: String,
    data: Option<ApiResponseData>,
}

impl ApiResponseData {
    /// Разбирает JSON-строку и возвращает данные или ошибку API.
    fn parse(json: &str) -> Result<Self, GameApiError> {
        let root: ApiRoot = serde_json::from_str(json)
            .map_err(|e| GameApiError::InvalidResponse(e.to_string()))?;
        if root.retcode != 0 {
            return Err(GameApiError::ApiError {
                retcode: root.retcode,
                message: root.message,
            });
        }
        root.data
            .ok_or_else(|| GameApiError::InvalidResponse("поле data отсутствует".into()))
    }
}

/// Преобразует одну запись API в доменный объект [`Wish`].
/// `None` — предмет не найден в справочнике (имя попадёт в `unknown_items`).
fn convert_record(
    record: &ApiWishRecord,
    reference: &ReferenceData,
    unknown_items: &mut Vec<String>,
) -> Result<Option<Wish>, domain::import::ImportError> {
    let code: u32 = record
        .gacha_type
        .parse()
        .map_err(|_| domain::import::ImportError::InvalidFile {
            format: "game-api".into(),
            reason: format!("неверный gacha_type «{}»", record.gacha_type),
        })?;

    let time = WishTime::parse(&record.time).map_err(|e| domain::import::ImportError::InvalidFile {
        format: "game-api".into(),
        reason: e.to_string(),
    })?;

    let Some(item) = reference.item_by_en_name(&record.name) else {
        unknown_items.push(record.name.clone());
        return Ok(None);
    };

    Ok(Some(Wish {
        id: WishId::new(&record.id),
        banner_code: BannerCode::new(code),
        item_id: item.id.clone(),
        item_kind: parse_item_type(&record.item_type).unwrap_or(item.kind),
        rarity: parse_rank_type(&record.rank_type).unwrap_or(item.rarity),
        time,
    }))
}

/// Разбирает тип предмета из API.
fn parse_item_type(value: &str) -> Option<ItemKind> {
    match value {
        "Character" => Some(ItemKind::Character),
        "Weapon" => Some(ItemKind::Weapon),
        _ => None,
    }
}

/// Разбирает редкость из строки.
fn parse_rank_type(value: &str) -> Option<Rarity> {
    match value {
        "3" => Some(Rarity::Three),
        "4" => Some(Rarity::Four),
        "5" => Some(Rarity::Five),
        _ => None,
    }
}

#[cfg(test)]
#[path = "game_api.tests.rs"]
mod tests;
