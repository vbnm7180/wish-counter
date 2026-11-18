//! Онлайн-обновление справочника Genshin из репозитория Paimon.moe.

use domain::reference::{
    ItemInfo, ReferenceBuildError, ReferenceSnapshot, ReferenceSourceSpec, RemoteReference,
    SourceFiles,
};
use domain::wish::ItemKind;

use super::converter::{PaimonSources, convert};
use super::{REPOSITORY_NAME, REPOSITORY_OWNER, source_files};

/// Описание источника Paimon.moe для общего механизма обновления.
#[derive(Debug, Default, Clone, Copy)]
pub struct PaimonRemoteReference;

impl RemoteReference for PaimonRemoteReference {
    /// Репозиторий Paimon.moe и четыре нужных файла.
    fn source(&self) -> ReferenceSourceSpec {
        ReferenceSourceSpec {
            owner: REPOSITORY_OWNER.to_owned(),
            repository: REPOSITORY_NAME.to_owned(),
            files: [
                source_files::BANNERS,
                source_files::CHARACTERS,
                source_files::WEAPONS,
                source_files::NAMES_RU,
            ]
            .map(str::to_owned)
            .to_vec(),
        }
    }

    /// Переводит скачанные файлы в справочник.
    fn build(
        &self,
        files: &SourceFiles,
        revision: &str,
        generated_at: &str,
    ) -> Result<ReferenceSnapshot, ReferenceBuildError> {
        let sources = sources_from_files(files)?;
        convert(&sources, revision, generated_at)
            .map(|conversion| conversion.snapshot)
            .map_err(|error| ReferenceBuildError(error.to_string()))
    }

    /// Картинки Paimon.moe: `static/images/characters/<id>.png` и `static/images/weapons/<id>.png`.
    fn image_path(&self, item: &ItemInfo) -> Option<String> {
        let folder = match item.kind {
            ItemKind::Character => source_files::CHARACTER_IMAGES,
            ItemKind::Weapon => source_files::WEAPON_IMAGES,
        };
        Some(format!("{folder}/{}.png", item.id))
    }
}

/// Раскладывает скачанные файлы по полям [`PaimonSources`].
///
/// Используется и приложением, и инструментом `data-sync`.
pub fn sources_from_files(files: &SourceFiles) -> Result<PaimonSources, ReferenceBuildError> {
    let take = |path: &str| {
        files
            .get(path)
            .cloned()
            .ok_or_else(|| ReferenceBuildError(format!("не скачан файл {path}")))
    };
    Ok(PaimonSources {
        banners_js: take(source_files::BANNERS)?,
        characters_js: take(source_files::CHARACTERS)?,
        weapons_js: take(source_files::WEAPONS)?,
        names_ru_json: take(source_files::NAMES_RU)?,
    })
}

// Тесты лежат рядом, в файле `remote.tests.rs`.
// `#[cfg(test)]` — модуль компилируется только при `cargo test`;
// `#[path]` указывает файл явно, т.к. имя отличается от стандартного `tests.rs`.
#[cfg(test)]
#[path = "remote.tests.rs"]
mod tests;
