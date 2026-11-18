//! Проверка настоящего онлайн-обновления справочника (нужен интернет).
//!
//! Тест помечен `#[ignore]`: обычный `cargo test` его пропускает, потому что
//! результат зависит от сети и от состояния репозитория Paimon.moe.
//! Запуск вручную:
//! ```text
//! cargo test -p wish-counter --test reference_online -- --ignored
//! ```

use std::sync::Arc;

use application::ports::ReferenceDownloader;
use application::services::{
    GameCatalog, ReferenceService, ReferenceUpdater, UpdateMode, UpdateOutcome,
};
use domain::game::GameId;
use domain::reference::RemoteReference;
use game_genshin::paimon::remote::PaimonRemoteReference;
use game_genshin::{GENSHIN_GAME_ID, GenshinModule};
use infrastructure::reference_cache::JsonReferenceCache;
use infrastructure::remote::{GitHubReferenceDownloader, HttpClient};

/// Скачивает файлы Paimon.moe и собирает из них справочник.
#[test]
#[ignore = "нужен интернет"]
fn downloads_and_builds_paimon_reference() {
    let downloader = GitHubReferenceDownloader::new(HttpClient::new());
    let source = PaimonRemoteReference.source();

    let revision = downloader.latest_revision(&source).unwrap();
    let files = downloader.download(&source, &revision).unwrap();
    let snapshot = PaimonRemoteReference
        .build(&files, &revision, "2026-01-01 00:00:00")
        .unwrap();

    assert_eq!(snapshot.source_revision, revision);
    assert!(snapshot.items.len() > 300);
    assert!(snapshot.banners.len() > 100);
}

/// Полный путь обновления в пустой папке кэша: проверка ревизии и запись состояния.
#[test]
#[ignore = "нужен интернет"]
fn updater_checks_real_source() {
    let temp = tempfile::tempdir().unwrap();
    let catalog = Arc::new(GameCatalog::new().with_module(Arc::new(GenshinModule)));
    let cache = Arc::new(JsonReferenceCache::new(temp.path()));
    let references = Arc::new(ReferenceService::new(&catalog, cache.as_ref()));
    let updater = ReferenceUpdater::new(
        catalog,
        references,
        cache,
        Arc::new(GitHubReferenceDownloader::new(HttpClient::new())),
    );
    let genshin = GameId::new(GENSHIN_GAME_ID);

    let outcome = updater
        .update(&genshin, UpdateMode::Force, chrono::Utc::now().naive_utc())
        .unwrap();

    assert!(
        matches!(
            outcome,
            UpdateOutcome::Updated { .. } | UpdateOutcome::UpToDate
        ),
        "{outcome:?}"
    );
    assert!(updater.status(&genshin).unwrap().last_checked_at.is_some());
}
