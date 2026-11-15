# CLAUDE.md — инструкции для ИИ-агентов

Этот файл нужен, чтобы любой ИИ-агент мог продолжить работу над проектом.
Перед началом работы прочитайте: [README.md](README.md), [docs/architecture.md](docs/architecture.md),
[docs/roadmap.md](docs/roadmap.md), [docs/decisions.md](docs/decisions.md).

## О проекте

Wish Counter — portable-приложение для Windows, которое считает молитвы в гача-играх HoYoverse
(сначала Genshin Impact, потом HSR и ZZZ). Аналог Paimon.moe, StarRailStation, ZZZ.moe в одном приложении.
Стек: Rust (ядро) + Tauri 2 + Svelte 5/TypeScript (интерфейс) + SQLite (с этапа 3).

Владелец проекта **изучает Rust** по этому коду, общается на русском. Работа идёт пошагово, вместе с владельцем.

## Обязательные правила

1. **Комментарий у каждой функции, структуры, трейта, модуля** — на русском: что делает и зачем.
   Для неочевидных конструкций Rust добавлять пояснение (это учебный проект). Имена — на английском.
2. **SOLID и Clean Code:** функция делает одно дело, короткие функции, без магических чисел,
   зависимости внедряются через конструкторы, конкретные реализации создаются только в
   `crates/app-desktop/src/bootstrap.rs`.
3. **Слои:** `domain` ни от кого не зависит; `application` → `domain`; `infrastructure` → `application`, `domain`;
   `games/*` → `domain`; `app-desktop` → все. Не нарушать.
4. **Без `unsafe`**, lints workspace должны проходить без предупреждений.
5. **Безопасность:** не логировать и не сохранять `authkey`; не расширять права в `capabilities/` без необходимости;
   никаких методов, нарушающих пользовательское соглашение игр (чтение памяти, инъекции, прокси) — см. docs/security.md.
6. **Тесты — в отдельном файле рядом с кодом, в папке модуля.** Модуль с тестами `foo` лежит в папке `foo/`:
   `foo/foo.rs` (код) и `foo/foo.tests.rs` (тесты). Родитель подключает модуль через
   `#[path = "foo/foo.rs"] mod foo;`, а в конце `foo.rs` стоит `#[cfg(test)] #[path = "foo.tests.rs"] mod tests;`.
   Для корня крейта: `lib.rs` + `lib.tests.rs`. Модули без тестов остаются обычными файлами `foo.rs`.
   Новая логика домена — только с тестами. Фронтенд тоже будет покрыт unit-тестами (Vitest, см. docs/roadmap.md).
7. **Документация:** при изменении структуры обновлять docs/architecture.md; новые решения — в docs/decisions.md;
   статус этапов — в docs/roadmap.md; результаты проверок — в docs/security.md; **в конце работы дописать журнал ниже**.
8. Тексты интерфейса — только через словари `ui/src/lib/i18n/locales/` (оба языка), цвета — только через CSS-переменные тем.
9. **Стили компонентов — внутри `.svelte` в теге `<style>`** (стандартный подход Svelte: стили автоматически
   изолированы внутри компонента). Глобальные стили и цвета тем — только в `ui/src/lib/themes/`.

## Окружение и команды

- Rust установлен в `C:\Rust` (`RUSTUP_HOME=C:\Rust\rustup`, `CARGO_HOME=C:\Rust\cargo`), так как в пути профиля пользователя есть кириллица.
  Если `cargo` не найден в bash: `export PATH="/c/Rust/cargo/bin:$PATH" RUSTUP_HOME='C:\Rust\rustup' CARGO_HOME='C:\Rust\cargo'`.
- Проверка перед завершением работы:
  ```bash
  cargo test --workspace
  cargo clippy --workspace --all-targets   # 0 предупреждений
  npm run check                            # 0 ошибок
  npm run app:build                        # portable .exe собирается
  ```
- Версии npm `@tauri-apps/*` закреплены на `~2.11` и должны совпадать с крейтом `tauri` по major.minor.
- TypeScript закреплён на `^6`: `svelte-check` 4.x пока не поддерживает TypeScript 7.

## Напоминания для владельца

- 📌 На **этапе 6** попросить владельца прислать файл экспорта из Paimon.moe, чтобы сверить формат.
- Установщик VS Build Tools просил перезагрузку (2026-11-14). Сборка работает и без неё.

## Журнал работы

### 2026-11-14 — Этап 0–1: окружение и каркас
**Решения владельца:** SQLite вместо PostgreSQL; Tauri + Svelte/TS; комментарии на русском;
темы (тёмная/светлая с возможностью добавления); языки RU/EN.

**Сделано:**
1. Установлены VS Build Tools 2026 (VCTools) через winget и Rust 1.100 через официальный rustup-init в `C:\Rust`.
2. Создан Cargo workspace (`Cargo.toml`): крейты domain, application, infrastructure, games/genshin, app-desktop;
   общие lints (`unsafe_code=forbid`, `missing_docs=warn`, clippy all); release-профиль (LTO, strip, panic=abort).
3. `domain`: `GameId`, `GameInfo`, трейт `GameModule`; `AppSettings`, `ThemeId` (проверка a-z0-9-, ≤32), `Language` (ru/en).
4. `application`: порт `SettingsRepository` + `RepositoryError`; сервисы `GameCatalog`, `SettingsService`
   (повреждённые настройки → значения по умолчанию + warn).
5. `infrastructure`: `AppPaths`/`StorageMode` (Portable = `<exe>/data`, UserProfile = `%APPDATA%\WishCounter`),
   `JsonSettingsRepository` (атомарная запись), `init_logging` (файл с ежедневной ротацией, 7 файлов + консоль).
6. `game-genshin`: `GenshinModule`.
7. `app-desktop`: Tauri 2, команды `list_games`, `get_settings`, `update_settings`; `bootstrap.rs` как composition root;
   минимальные capabilities (`core:default`), строгий CSP; иконки сгенерированы из `assets/branding/icon.svg`.
8. Интерфейс `ui/`: Svelte 5 (руны), TypeScript, Vite 8. Меню, страницы Home/Game/Settings, `BannerCard` (заглушки),
   реестр тем + `dark.css`/`light.css`, локализация с проверкой полноты словарей на уровне типов,
   `BrowserBackend` для верстки в браузере без Rust.
9. Документация: README, docs/architecture, development, roadmap, decisions (ADR-001…008), security, rust-notes.

**Проверено:** 23 Rust-теста проходят; clippy без предупреждений; `svelte-check` без ошибок;
`npm run app:build` → `target/release/wish-counter.exe` (≈5 МБ) запускается, создаёт `data/logs/`,
основной процесс ≈31 МБ RAM (процессы WebView2 не измерялись).

### 2026-11-15 — Замечания ревью владельца к этапу 1
**Раскладка тестов Rust (итог двух ревью):** модуль с тестами — папка `foo/` с файлами `foo.rs` и `foo.tests.rs`,
подключение через атрибут `#[path]` (ADR-009). Перенесены 7 модулей: domain/{game, settings},
application/services/{game_catalog, settings_service}, infrastructure/{paths, settings/json_settings_repository},
games/genshin (`lib.tests.rs`). Промежуточный вариант `foo.rs` + `foo/tests.rs` владелец отклонил:
код и тесты оказывались в разных папках.

**Стили Svelte:** стили выносили в `*.module.css` (CSS Modules), затем по решению владельца вернули `<style>`
внутрь компонентов как стандартный подход фреймворка (ADR-010). Файлы `*.module.css` удалены.

**Добавлено:** `.gitattributes` (LF для всех текстовых файлов); пункт про unit-тесты фронтенда (Vitest) в roadmap.

**Проверено:** 23 Rust-теста, clippy 0 предупреждений, svelte-check 0 ошибок, portable .exe собирается.

**Следующий шаг:** этап 2 — домен молитв и расчёт гаранта Genshin.
