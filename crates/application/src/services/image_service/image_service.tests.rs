//! Модульные тесты для `image_service.rs`.
//!
//! Подключаются в `image_service.rs` строками
//! `#[cfg(test)] #[path = "image_service.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use domain::game::{GameInfo, GameModule};
use domain::pity::rules::PoolDefinition;
use domain::reference::{
    ItemInfo, LocalizedText, ReferenceBuildError, ReferenceSnapshot, ReferenceSourceSpec,
    RemoteReference, SourceFiles,
};
use domain::wish::{ItemKind, Rarity};

use super::*;
use crate::ports::DownloadError;

/// Предмет, который есть в справочнике тестовой игры.
const KNOWN_ITEM: &str = "hu_tao";

/// Корректный «PNG»: сигнатура и пара байт данных.
fn png() -> Vec<u8> {
    let mut bytes = PNG_SIGNATURE.to_vec();
    bytes.extend_from_slice(&[1, 2, 3]);
    bytes
}

/// Тестовый источник: картинка лежит по пути `images/<id>.png`.
struct FakeRemote;

impl RemoteReference for FakeRemote {
    fn source(&self) -> ReferenceSourceSpec {
        ReferenceSourceSpec {
            owner: "owner".into(),
            repository: "repo".into(),
            files: vec![],
        }
    }

    fn build(
        &self,
        _files: &SourceFiles,
        _revision: &str,
        _generated_at: &str,
    ) -> Result<ReferenceSnapshot, ReferenceBuildError> {
        unimplemented!("в тестах картинок справочник не собирается")
    }

    fn image_path(&self, item: &ItemInfo) -> Option<String> {
        Some(format!("images/{}.png", item.id))
    }
}

/// Тестовая игра с одним предметом в справочнике.
struct FakeGame;

impl GameModule for FakeGame {
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new("fake"),
            display_order: 1,
        }
    }

    fn pools(&self) -> Vec<PoolDefinition> {
        Vec::new()
    }

    fn embedded_reference(&self) -> ReferenceSnapshot {
        ReferenceSnapshot {
            source_revision: "rev".into(),
            generated_at: "2026-11-18 00:00:00".into(),
            items: vec![ItemInfo {
                id: ItemId::new(KNOWN_ITEM),
                kind: ItemKind::Character,
                rarity: Rarity::Five,
                name: LocalizedText {
                    en: "Hu Tao".into(),
                    ru: "Ху Тао".into(),
                },
            }],
            banners: vec![],
        }
    }

    fn remote_reference(&self) -> Option<Arc<dyn RemoteReference>> {
        Some(Arc::new(FakeRemote))
    }

    fn icon_png(&self) -> Option<&'static [u8]> {
        Some(b"icon")
    }
}

/// Загрузчик: отдаёт заданный ответ и считает запросы, запоминает последний путь.
struct FakeDownloader {
    response: Result<Vec<u8>, DownloadError>,
    requests: AtomicUsize,
    last_path: Mutex<Option<String>>,
}

impl FakeDownloader {
    /// Загрузчик, который всегда отвечает `response`.
    fn answering(response: Result<Vec<u8>, DownloadError>) -> Self {
        Self {
            response,
            requests: AtomicUsize::new(0),
            last_path: Mutex::new(None),
        }
    }
}

impl ImageDownloader for FakeDownloader {
    fn download_image(
        &self,
        _source: &ReferenceSourceSpec,
        revision: &str,
        path: &str,
    ) -> Result<Vec<u8>, DownloadError> {
        assert_eq!(
            revision, "rev",
            "картинка скачивается из ревизии справочника"
        );
        self.requests.fetch_add(1, Ordering::SeqCst);
        *self.last_path.lock().unwrap() = Some(path.to_owned());
        self.response.clone()
    }
}

/// Кэш картинок в памяти.
#[derive(Default)]
struct InMemoryImages(Mutex<HashMap<(GameId, ItemId), Vec<u8>>>);

impl ImageCache for InMemoryImages {
    fn load(&self, game_id: &GameId, item_id: &ItemId) -> Result<Option<Vec<u8>>, RepositoryError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .get(&(game_id.clone(), item_id.clone()))
            .cloned())
    }

    fn save(
        &self,
        game_id: &GameId,
        item_id: &ItemId,
        bytes: &[u8],
    ) -> Result<(), RepositoryError> {
        self.0
            .lock()
            .unwrap()
            .insert((game_id.clone(), item_id.clone()), bytes.to_vec());
        Ok(())
    }
}

/// Сервис и доступ к подменам.
struct Setup {
    service: ImageService,
    cache: Arc<InMemoryImages>,
    downloader: Arc<FakeDownloader>,
}

/// Собирает сервис с тестовой игрой и загрузчиком.
fn setup(downloader: FakeDownloader) -> Setup {
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(FakeGame)));
    let references = Arc::new(ReferenceService::from_catalog(&catalog));
    let cache = Arc::new(InMemoryImages::default());
    let downloader = Arc::new(downloader);
    let service = ImageService::new(catalog, references, cache.clone(), downloader.clone());
    Setup {
        service,
        cache,
        downloader,
    }
}

/// Идентификатор тестовой игры.
fn fake() -> GameId {
    GameId::new("fake")
}

/// Количество обращений к загрузчику.
fn requests(setup: &Setup) -> usize {
    setup.downloader.requests.load(Ordering::SeqCst)
}

#[test]
fn downloads_known_item_and_caches_it() {
    let setup = setup(FakeDownloader::answering(Ok(png())));
    let hu_tao = ItemId::new(KNOWN_ITEM);

    let image = setup.service.image(&fake(), &hu_tao).unwrap();

    assert_eq!(image, Some(png()));
    assert_eq!(setup.cache.load(&fake(), &hu_tao).unwrap(), Some(png()));
    assert_eq!(
        setup.downloader.last_path.lock().unwrap().as_deref(),
        Some("images/hu_tao.png")
    );
}

/// Второй запрос отдаётся из кэша, без сети.
#[test]
fn cached_image_is_served_without_download() {
    let setup = setup(FakeDownloader::answering(Ok(png())));
    let hu_tao = ItemId::new(KNOWN_ITEM);

    setup.service.image(&fake(), &hu_tao).unwrap();
    setup.service.image(&fake(), &hu_tao).unwrap();

    assert_eq!(requests(&setup), 1);
}

/// Картинку предмета, которого нет в справочнике, не скачиваем вовсе.
#[test]
fn unknown_item_is_not_requested() {
    let setup = setup(FakeDownloader::answering(Ok(png())));

    let image = setup
        .service
        .image(&fake(), &ItemId::new("../../secret"))
        .unwrap();

    assert_eq!(image, None);
    assert_eq!(requests(&setup), 0);
}

/// Не PNG (например, HTML-страница ошибки) не сохраняется.
#[test]
fn non_png_response_is_rejected() {
    let setup = setup(FakeDownloader::answering(Ok(b"<html>".to_vec())));
    let hu_tao = ItemId::new(KNOWN_ITEM);

    assert_eq!(setup.service.image(&fake(), &hu_tao).unwrap(), None);
    assert_eq!(setup.cache.load(&fake(), &hu_tao).unwrap(), None);
}

/// После неудачи картинка не запрашивается повторно в этом сеансе.
#[test]
fn failed_download_is_not_retried_in_same_session() {
    let setup = setup(FakeDownloader::answering(Err(DownloadError(
        "нет сети".into(),
    ))));
    let hu_tao = ItemId::new(KNOWN_ITEM);

    assert_eq!(setup.service.image(&fake(), &hu_tao).unwrap(), None);
    assert_eq!(setup.service.image(&fake(), &hu_tao).unwrap(), None);

    assert_eq!(requests(&setup), 1);
}

#[test]
fn unknown_game_is_reported() {
    let setup = setup(FakeDownloader::answering(Ok(png())));

    let result = setup
        .service
        .image(&GameId::new("missing"), &ItemId::new(KNOWN_ITEM));

    assert!(matches!(result, Err(ImageError::UnknownGame(_))));
}

#[test]
fn game_icon_comes_from_game_module() {
    let setup = setup(FakeDownloader::answering(Ok(png())));

    assert_eq!(
        setup.service.game_icon(&fake()).unwrap(),
        Some(b"icon".to_vec())
    );
    assert!(matches!(
        setup.service.game_icon(&GameId::new("missing")),
        Err(ImageError::UnknownGame(_))
    ));
    assert_eq!(requests(&setup), 0, "иконка игры не скачивается");
}

/// Докачка: известный предмет скачивается, неизвестный пропускается без ошибки.
#[test]
fn prefetch_downloads_known_items() {
    let setup = setup(FakeDownloader::answering(Ok(png())));

    let cached = setup.service.prefetch(
        &fake(),
        &[ItemId::new(KNOWN_ITEM), ItemId::new("unknown_item")],
    );

    assert_eq!(cached, 1);
    assert_eq!(requests(&setup), 1);
}
