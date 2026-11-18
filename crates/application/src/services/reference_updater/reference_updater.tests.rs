//! Модульные тесты для `reference_updater.rs`.
//!
//! Подключаются в `reference_updater.rs` строками
//! `#[cfg(test)] #[path = "reference_updater.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Сеть не используется: загрузчик, кэш и игра подменены тестовыми реализациями.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use domain::game::{GameInfo, GameModule};
use domain::pity::rules::{PoolDefinition, PoolId};
use domain::reference::{
    BannerInfo, ItemInfo, LocalizedText, ReferenceBuildError, ReferenceSourceSpec, ScheduleTime,
    SourceFiles,
};
use domain::wish::{ItemId, ItemKind, Rarity, WishTime};

use super::*;

/// Ревизия встроенного справочника тестовой игры.
const EMBEDDED_REVISION: &str = "rev-1";
/// Имя единственного «скачиваемого» файла.
const DATA_FILE: &str = "data.txt";
/// Содержимое файла, которое тестовый источник не может разобрать.
const BROKEN_CONTENT: &str = "broken";

/// Справочник с `item_count` предметами и одним баннером.
fn snapshot(revision: &str, generated_at: &str, item_count: usize) -> ReferenceSnapshot {
    let time = ScheduleTime {
        time: WishTime::parse("2026-01-01 00:00:00").unwrap(),
        asia_based: false,
    };
    ReferenceSnapshot {
        source_revision: revision.into(),
        generated_at: generated_at.into(),
        items: (0..item_count)
            .map(|index| ItemInfo {
                id: ItemId::new(format!("item_{index}")),
                kind: ItemKind::Character,
                rarity: Rarity::Five,
                name: LocalizedText {
                    en: "Item".into(),
                    ru: "Предмет".into(),
                },
            })
            .collect(),
        banners: vec![BannerInfo {
            pool_id: PoolId::new("character"),
            name: "Banner".into(),
            short_name: "Banner".into(),
            version: None,
            start: time.clone(),
            end: time,
            featured: vec![],
            featured_rare: vec![],
        }],
    }
}

/// Тестовый источник: файл `data.txt` содержит число предметов.
struct FakeRemote;

impl RemoteReference for FakeRemote {
    fn source(&self) -> ReferenceSourceSpec {
        ReferenceSourceSpec {
            owner: "owner".into(),
            repository: "repo".into(),
            files: vec![DATA_FILE.into()],
        }
    }

    fn build(
        &self,
        files: &SourceFiles,
        revision: &str,
        generated_at: &str,
    ) -> Result<ReferenceSnapshot, ReferenceBuildError> {
        let count = files[DATA_FILE]
            .parse()
            .map_err(|_| ReferenceBuildError("не число".into()))?;
        Ok(snapshot(revision, generated_at, count))
    }
}

/// Тестовая игра с онлайн-обновлением (`with_remote = true`) или без него.
struct FakeGame {
    id: &'static str,
    with_remote: bool,
}

impl GameModule for FakeGame {
    fn info(&self) -> GameInfo {
        GameInfo {
            id: GameId::new(self.id),
            display_order: 1,
        }
    }

    fn pools(&self) -> Vec<PoolDefinition> {
        Vec::new()
    }

    fn embedded_reference(&self) -> ReferenceSnapshot {
        snapshot(EMBEDDED_REVISION, "2026-11-01 00:00:00", 4)
    }

    fn remote_reference(&self) -> Option<Arc<dyn RemoteReference>> {
        self.with_remote
            .then(|| Arc::new(FakeRemote) as Arc<dyn RemoteReference>)
    }
}

/// Тестовый загрузчик: отдаёт заданные ревизию и содержимое, считает обращения.
#[derive(Default)]
struct FakeDownloader {
    revision: Mutex<String>,
    content: Mutex<String>,
    offline: bool,
    downloads: AtomicUsize,
}

impl FakeDownloader {
    /// Загрузчик, у которого «в сети» лежит ревизия `revision` с `content`.
    fn serving(revision: &str, content: &str) -> Self {
        Self {
            revision: Mutex::new(revision.into()),
            content: Mutex::new(content.into()),
            ..Self::default()
        }
    }

    /// Загрузчик без сети: любой запрос — ошибка.
    fn offline() -> Self {
        Self {
            offline: true,
            ..Self::default()
        }
    }
}

impl ReferenceDownloader for FakeDownloader {
    fn latest_revision(&self, _source: &ReferenceSourceSpec) -> Result<String, DownloadError> {
        if self.offline {
            return Err(DownloadError("нет сети".into()));
        }
        Ok(self.revision.lock().unwrap().clone())
    }

    fn download(
        &self,
        _source: &ReferenceSourceSpec,
        _revision: &str,
    ) -> Result<SourceFiles, DownloadError> {
        self.downloads.fetch_add(1, Ordering::SeqCst);
        let content = self.content.lock().unwrap().clone();
        Ok(HashMap::from([(DATA_FILE.to_owned(), content)]))
    }
}

/// Кэш в памяти.
#[derive(Default)]
struct InMemoryCache {
    snapshots: Mutex<HashMap<GameId, ReferenceSnapshot>>,
    checked: Mutex<HashMap<GameId, NaiveDateTime>>,
}

impl ReferenceCache for InMemoryCache {
    fn load_snapshot(
        &self,
        game_id: &GameId,
    ) -> Result<Option<ReferenceSnapshot>, RepositoryError> {
        Ok(self.snapshots.lock().unwrap().get(game_id).cloned())
    }

    fn save_snapshot(
        &self,
        game_id: &GameId,
        snapshot: &ReferenceSnapshot,
    ) -> Result<(), RepositoryError> {
        self.snapshots
            .lock()
            .unwrap()
            .insert(game_id.clone(), snapshot.clone());
        Ok(())
    }

    fn last_checked_at(&self, game_id: &GameId) -> Result<Option<NaiveDateTime>, RepositoryError> {
        Ok(self.checked.lock().unwrap().get(game_id).copied())
    }

    fn set_last_checked_at(
        &self,
        game_id: &GameId,
        at: NaiveDateTime,
    ) -> Result<(), RepositoryError> {
        self.checked.lock().unwrap().insert(game_id.clone(), at);
        Ok(())
    }
}

/// Всё, что нужно тесту: сервис обновления и доступ к подменам.
struct Setup {
    updater: ReferenceUpdater,
    references: Arc<ReferenceService>,
    cache: Arc<InMemoryCache>,
    downloader: Arc<FakeDownloader>,
}

/// Собирает сервис с играми `fake` (с обновлением) и `offline` (без него).
fn setup(downloader: FakeDownloader) -> Setup {
    let catalog = Arc::new(
        GameCatalog::new()
            .with_module(Arc::new(FakeGame {
                id: "fake",
                with_remote: true,
            }))
            .with_module(Arc::new(FakeGame {
                id: "offline",
                with_remote: false,
            })),
    );
    let cache = Arc::new(InMemoryCache::default());
    let downloader = Arc::new(downloader);
    let references = Arc::new(ReferenceService::new(&catalog, cache.as_ref()));
    let updater = ReferenceUpdater::new(
        catalog,
        references.clone(),
        cache.clone(),
        downloader.clone(),
    );
    Setup {
        updater,
        references,
        cache,
        downloader,
    }
}

/// Время `ГГГГ-ММ-ДД чч:мм:сс` для тестов.
fn time(text: &str) -> NaiveDateTime {
    WishTime::parse(text).unwrap().to_date_time()
}

/// Идентификатор тестовой игры с обновлением.
fn fake() -> GameId {
    GameId::new("fake")
}

/// Текущая ревизия справочника игры `fake`.
fn current_revision(setup: &Setup) -> String {
    setup
        .references
        .reference(&fake())
        .unwrap()
        .snapshot()
        .source_revision
        .clone()
}

#[test]
fn new_revision_is_downloaded_cached_and_applied() {
    let setup = setup(FakeDownloader::serving("rev-2", "5"));
    let now = time("2026-11-17 10:00:00");

    let outcome = setup
        .updater
        .update(&fake(), UpdateMode::Force, now)
        .unwrap();

    assert_eq!(
        outcome,
        UpdateOutcome::Updated {
            revision: "rev-2".into()
        }
    );
    assert_eq!(current_revision(&setup), "rev-2");
    assert_eq!(
        setup
            .cache
            .load_snapshot(&fake())
            .unwrap()
            .unwrap()
            .items
            .len(),
        5
    );
    assert_eq!(setup.cache.last_checked_at(&fake()).unwrap(), Some(now));
}

/// Если ревизия не изменилась, файлы не скачиваются.
#[test]
fn same_revision_is_up_to_date_without_download() {
    let setup = setup(FakeDownloader::serving(EMBEDDED_REVISION, "5"));

    let outcome = setup
        .updater
        .update(&fake(), UpdateMode::Force, time("2026-11-17 10:00:00"))
        .unwrap();

    assert_eq!(outcome, UpdateOutcome::UpToDate);
    assert_eq!(setup.downloader.downloads.load(Ordering::SeqCst), 0);
}

/// Автоматическая проверка — не чаще раза в сутки.
#[test]
fn automatic_check_runs_at_most_once_a_day() {
    let setup = setup(FakeDownloader::serving(EMBEDDED_REVISION, "5"));
    setup
        .updater
        .update(&fake(), UpdateMode::IfDue, time("2026-11-17 10:00:00"))
        .unwrap();

    let soon = setup
        .updater
        .update(&fake(), UpdateMode::IfDue, time("2026-11-18 09:59:59"))
        .unwrap();
    let next_day = setup
        .updater
        .update(&fake(), UpdateMode::IfDue, time("2026-11-18 10:00:00"))
        .unwrap();

    assert_eq!(soon, UpdateOutcome::NotDue);
    assert_eq!(next_day, UpdateOutcome::UpToDate);
}

/// Кнопка «Обновить» проверяет сразу, даже если сутки не прошли.
#[test]
fn forced_check_ignores_interval() {
    let setup = setup(FakeDownloader::serving(EMBEDDED_REVISION, "5"));
    setup
        .updater
        .update(&fake(), UpdateMode::IfDue, time("2026-11-17 10:00:00"))
        .unwrap();

    let outcome = setup
        .updater
        .update(&fake(), UpdateMode::Force, time("2026-11-17 10:05:00"))
        .unwrap();

    assert_eq!(outcome, UpdateOutcome::UpToDate);
}

/// Без сети — ошибка, справочник прежний, время проверки не записано (повторим позже).
#[test]
fn network_error_keeps_reference_and_does_not_mark_checked() {
    let setup = setup(FakeDownloader::offline());

    let result = setup
        .updater
        .update(&fake(), UpdateMode::IfDue, time("2026-11-17 10:00:00"));

    assert!(matches!(result, Err(UpdateError::Download(_))));
    assert_eq!(current_revision(&setup), EMBEDDED_REVISION);
    assert_eq!(setup.cache.last_checked_at(&fake()).unwrap(), None);
}

#[test]
fn unparsable_data_is_rejected() {
    let setup = setup(FakeDownloader::serving("rev-2", BROKEN_CONTENT));

    let result = setup
        .updater
        .update(&fake(), UpdateMode::Force, time("2026-11-17 10:00:00"));

    assert!(matches!(result, Err(UpdateError::InvalidData(_))));
    assert_eq!(current_revision(&setup), EMBEDDED_REVISION);
}

/// Резкое уменьшение числа предметов (4 → 1) считается испорченными данными.
#[test]
fn suspicious_shrink_is_rejected() {
    let setup = setup(FakeDownloader::serving("rev-2", "1"));

    let result = setup
        .updater
        .update(&fake(), UpdateMode::Force, time("2026-11-17 10:00:00"));

    assert!(matches!(result, Err(UpdateError::InvalidData(_))));
    assert!(setup.cache.load_snapshot(&fake()).unwrap().is_none());
}

#[test]
fn game_without_remote_source_is_unsupported() {
    let setup = setup(FakeDownloader::serving("rev-2", "5"));

    let outcome = setup
        .updater
        .update(
            &GameId::new("offline"),
            UpdateMode::Force,
            time("2026-11-17 10:00:00"),
        )
        .unwrap();

    assert_eq!(outcome, UpdateOutcome::Unsupported);
}

#[test]
fn update_all_due_checks_every_game() {
    let setup = setup(FakeDownloader::serving("rev-2", "5"));

    let results = setup.updater.update_all_due(time("2026-11-17 10:00:00"));

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(_, result)| result.is_ok()));
}

#[test]
fn status_includes_last_check_time() {
    let setup = setup(FakeDownloader::serving(EMBEDDED_REVISION, "5"));
    setup
        .updater
        .update(&fake(), UpdateMode::Force, time("2026-11-17 10:00:00"))
        .unwrap();

    let status = setup.updater.status(&fake()).unwrap();

    assert_eq!(
        status.last_checked_at.as_deref(),
        Some("2026-11-17 10:00:00")
    );
    assert_eq!(status.summary.source_revision, EMBEDDED_REVISION);
}

// ---------- Загрузка при запуске ----------

/// При запуске используется скачанный справочник, если он новее встроенного.
#[test]
fn newer_downloaded_reference_is_used_on_start() {
    let cache = InMemoryCache::default();
    cache
        .save_snapshot(
            &fake(),
            &snapshot("rev-downloaded", "2026-11-10 00:00:00", 4),
        )
        .unwrap();
    let catalog = GameCatalog::new().with_module(Arc::new(FakeGame {
        id: "fake",
        with_remote: true,
    }));

    let references = ReferenceService::new(&catalog, &cache);

    let revision = references
        .reference(&fake())
        .unwrap()
        .snapshot()
        .source_revision
        .clone();
    assert_eq!(revision, "rev-downloaded");
}

/// Скачанный справочник старше встроенного (вышла новая версия приложения) — игнорируется.
#[test]
fn older_downloaded_reference_is_ignored_on_start() {
    let cache = InMemoryCache::default();
    cache
        .save_snapshot(&fake(), &snapshot("rev-old", "2026-10-01 00:00:00", 4))
        .unwrap();
    let catalog = GameCatalog::new().with_module(Arc::new(FakeGame {
        id: "fake",
        with_remote: true,
    }));

    let references = ReferenceService::new(&catalog, &cache);

    let revision = references
        .reference(&fake())
        .unwrap()
        .snapshot()
        .source_revision
        .clone();
    assert_eq!(revision, EMBEDDED_REVISION);
}
