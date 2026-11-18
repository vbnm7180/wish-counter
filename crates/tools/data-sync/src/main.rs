//! # data-sync — обновление встроенного справочника
//!
//! Запуск из корня проекта:
//! ```text
//! cargo run -p data-sync
//! ```
//! Что делает:
//! 1. узнаёт последнюю ревизию репозитория Paimon.moe на GitHub;
//! 2. скачивает файлы баннеров, персонажей, оружия и русских названий этой ревизии;
//! 3. переводит их в формат справочника и записывает
//!    `crates/games/genshin/data/reference.json`.
//!
//! После запуска нужно пересобрать приложение и закоммитить файл.

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use application::ports::ReferenceDownloader;
use domain::reference::RemoteReference;
use game_genshin::paimon::converter::{Conversion, convert};
use game_genshin::paimon::remote::{PaimonRemoteReference, sources_from_files};
use infrastructure::remote::{GitHubReferenceDownloader, HttpClient};

/// Формат отметки времени создания снимка.
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Точка входа: выводит ошибку и возвращает ненулевой код при сбое.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Ошибка: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Скачивает, переводит и сохраняет справочник.
fn run() -> Result<(), Box<dyn Error>> {
    // Тот же загрузчик и тот же список файлов, что использует приложение.
    let downloader = GitHubReferenceDownloader::new(HttpClient::new());
    let source = PaimonRemoteReference.source();

    let revision = downloader.latest_revision(&source)?;
    println!("Ревизия Paimon.moe: {revision}");
    println!("Скачивание: {}", source.files.join(", "));
    let files = downloader.download(&source, &revision)?;
    let sources = sources_from_files(&files)?;

    let generated_at = chrono::Utc::now().format(TIMESTAMP_FORMAT).to_string();
    let conversion = convert(&sources, &revision, &generated_at)?;
    report(&conversion);
    write_snapshot(&conversion)
}

/// Печатает итоги перевода и предупреждения.
fn report(conversion: &Conversion) {
    let snapshot = &conversion.snapshot;
    println!(
        "Предметов: {}, баннеров: {}",
        snapshot.items.len(),
        snapshot.banners.len()
    );
    if !conversion.unknown_featured_ids.is_empty() {
        println!(
            "Внимание: в баннерах есть предметы без описания: {}",
            conversion.unknown_featured_ids.join(", ")
        );
    }
}

/// Записывает справочник в `crates/games/genshin/data/reference.json`.
fn write_snapshot(conversion: &Conversion) -> Result<(), Box<dyn Error>> {
    // env!("CARGO_MANIFEST_DIR") — папка этого крейта на момент компиляции.
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/genshin/data/reference.json");
    let json = serde_json::to_string_pretty(&conversion.snapshot)?;
    std::fs::write(&path, json + "\n")?;
    println!("Записано: {}", path.display());
    Ok(())
}
