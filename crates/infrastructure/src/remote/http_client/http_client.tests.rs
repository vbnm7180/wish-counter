//! Модульные тесты для `http_client.rs`.
//!
//! Подключаются в `http_client.rs` строками `#[cfg(test)] #[path = "http_client.tests.rs"] mod tests;`.
//! Это дочерний модуль, поэтому через `use super::*` он видит и приватные элементы.
//!
//! Тесты не ходят в сеть: проверяется только фильтр адресов.

use super::*;

#[test]
fn allows_https_github_hosts() {
    assert!(check_url("https://raw.githubusercontent.com/a/b/c/d.js").is_ok());
    assert!(check_url("https://api.github.com/repos/a/b/commits/main").is_ok());
}

#[test]
fn rejects_plain_http() {
    assert!(check_url("http://raw.githubusercontent.com/a.js").is_err());
}

#[test]
fn rejects_other_hosts() {
    assert!(check_url("https://example.com/a.js").is_err());
    assert!(check_url("https://raw.githubusercontent.com.evil.com/a.js").is_err());
}

/// Попытки подставить другой хост через `user@host` или порт отклоняются.
#[test]
fn rejects_userinfo_and_ports() {
    assert!(check_url("https://raw.githubusercontent.com@evil.com/a.js").is_err());
    assert!(check_url("https://api.github.com:8443/a").is_err());
}

/// Запрос к запрещённому адресу отклоняется ещё до обращения к сети.
#[test]
fn get_text_rejects_forbidden_url_without_network() {
    let result = HttpClient::new().get_text("https://example.com/");
    assert!(matches!(result, Err(HttpError::UrlNotAllowed(_))));
}
