//! Модульные тесты для `github.rs`.
//!
//! Подключаются в `github.rs` строками `#[cfg(test)] #[path = "github.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Тесты не ходят в сеть.

use super::*;

#[test]
fn accepts_full_commit_hash() {
    assert!(validate_revision("0123456789abcdef0123456789abcdef01234567").is_ok());
}

#[test]
fn rejects_malformed_revisions() {
    for revision in [
        "main",
        "",
        "../../etc",
        "0123456789abcdef0123456789abcdef0123456g",
    ] {
        assert!(validate_revision(revision).is_err(), "{revision}");
    }
}

/// Некорректная ревизия отклоняется до обращения к сети.
#[test]
fn fetch_text_rejects_bad_revision_without_network() {
    let repository = GitHubRepository::new("owner", "repo");

    let result = repository.fetch_text(&HttpClient::new(), "main/../../x", "file.js");

    assert!(matches!(result, Err(GitHubError::InvalidResponse(_))));
}
