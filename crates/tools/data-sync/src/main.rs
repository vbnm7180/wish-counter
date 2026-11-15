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

use game_genshin::paimon::converter::{Conversion, PaimonSources, convert};
use game_genshin::paimon::{REPOSITORY_NAME, REPOSITORY_OWNER, source_files};
use infrastructure::remote::{GitHubRepository, HttpClient};

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
    let http = HttpClient::new();
    let repository = GitHubRepository::new(REPOSITORY_OWNER, REPOSITORY_NAME);

    let revision = repository.latest_revision(&http)?;
    println!("Ревизия Paimon.moe: {revision}");
    let fetch = |path: &str| {
        println!("  скачивание {path}");
        repository.fetch_text(&http, &revision, path)
    };
    let sources = PaimonSources {
        banners_js: fetch(source_files::BANNERS)?,
        characters_js: fetch(source_files::CHARACTERS)?,
        weapons_js: fetch(source_files::WEAPONS)?,
        names_ru_json: fetch(source_files::NAMES_RU)?,
    };

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
