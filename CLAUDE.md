# CLAUDE.md — инструкции для ИИ-агентов

Этот файл нужен, чтобы любой ИИ-агент мог продолжить работу над проектом.
Перед началом работы прочитайте: [README.md](README.md), [docs/architecture.md](docs/architecture.md),
[docs/roadmap.md](docs/roadmap.md), [docs/decisions.md](docs/decisions.md).

## О проекте

Wish Counter — portable-приложение для Windows, которое считает молитвы в гача-играх HoYoverse
(сначала Genshin Impact, потом HSR и ZZZ). Аналог Paimon.moe, StarRailStation, ZZZ.moe в одном приложении.
Стек: Rust (ядро) + Tauri 2 + Svelte 5/TypeScript (интерфейс) + SQLite (rusqlite, bundled).

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
5. **Portable:** приложение пишет только в `data/` рядом с .exe (включая WebView2 — ADR-017). Картинки и WebView2
   в .exe не встраивать (ADR-018).
6. **Безопасность:** не логировать и не сохранять `authkey`; не расширять права в `capabilities/` без необходимости;
   никаких методов, нарушающих пользовательское соглашение игр (чтение памяти, инъекции, прокси) — см. docs/security.md.
7. **Тесты — в отдельном файле рядом с кодом, в папке модуля.** Модуль с тестами `foo` лежит в папке `foo/`:
   `foo/foo.rs` (код) и `foo/foo.tests.rs` (тесты). Родитель подключает модуль через
   `#[path = "foo/foo.rs"] mod foo;`, а в конце `foo.rs` стоит `#[cfg(test)] #[path = "foo.tests.rs"] mod tests;`.
   Для корня крейта: `lib.rs` + `lib.tests.rs`. Модули без тестов остаются обычными файлами `foo.rs`.
   Новая логика домена — только с тестами. Фронтенд тоже будет покрыт unit-тестами (Vitest, см. docs/roadmap.md).
8. **Документация:** при изменении структуры обновлять docs/architecture.md; новые решения — в docs/decisions.md;
   статус этапов — в docs/roadmap.md; результаты проверок — в docs/security.md; **в конце работы дописать журнал ниже**.
9. Тексты интерфейса — только через словари `ui/src/lib/i18n/locales/` (оба языка), цвета — только через CSS-переменные тем.
10. **Стили компонентов — внутри `.svelte` в теге `<style>`** (стандартный подход Svelte: стили автоматически
   изолированы внутри компонента). Глобальные стили и цвета тем — только в `ui/src/lib/themes/`.
11. **Миграции БД:** выпущенные миграции не менять; изменения схемы — только новым файлом `sql/000N_*.sql`
   (docs/development.md). SQL — только с параметрами, без подстановки строк.

## Окружение и команды

- Rust установлен в `C:\Rust` (`RUSTUP_HOME=C:\Rust\rustup`, `CARGO_HOME=C:\Rust\cargo`), так как в пути профиля пользователя есть кириллица.
  Если `cargo` не найден в bash: `export PATH="/c/Rust/cargo/bin:$PATH" RUSTUP_HOME='C:\Rust\rustup' CARGO_HOME='C:\Rust\cargo'`.
- Проверка перед завершением работы:
  ```bash
  cargo test --workspace
  cargo clippy --workspace --all-targets   # 0 предупреждений
  npm run check                            # 0 ошибок
  npm test                                 # тесты интерфейса (Vitest)
  npm run app:build                        # portable .exe собирается
  ```
- Версии npm `@tauri-apps/*` закреплены на `~2.11` и должны совпадать с крейтом `tauri` по major.minor.
- TypeScript закреплён на `^6`: `svelte-check` 4.x пока не поддерживает TypeScript 7.

## Напоминания для владельца

- 📌 На **этапе 6** попросить владельца прислать файл экспорта из Paimon.moe, чтобы сверить формат.
- 📌 Ближе к **этапу 10** напомнить владельцу обсудить формат дистрибутива (ZIP с одним .exe и т.п.).
- 📌 Отложено, но **обязательно**: шанс нужного 4★ (50%/75%) — см. «Отложено» в docs/roadmap.md.
- Правила гаранта Genshin и решения владельца по ним — в docs/genshin-rules.md. Перед изменением расчёта сверяться с ним.
- Справочник `crates/games/genshin/data/reference.json` — только через `cargo run -p data-sync`, не вручную (ADR-020).
  Данные Paimon.moe под MIT: атрибуция в THIRD_PARTY_NOTICES.md и в настройках приложения — не удалять.
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

### 2026-11-15 — Этап 2: домен молитв и расчёт гаранта
**Решения владельца:** счётчик 4★ не сбрасывается при 5★; шанс нужного 4★ отложен (досчитать позже);
баннер новичка считается отдельно; показывать шанс персонажа с баннера и правило «3 проигрыша → гарант» (Capturing Radiance);
примогемы — по игре в целом и по каждому баннеру.

**Сделано:**
1. Правила сверены по гайдам для версии 7.x и записаны в docs/genshin-rules.md.
2. `domain::wish`: `Wish`, `WishId` (строка — 19 цифр не помещаются в JS number), `ItemId`, `BannerCode`, `ItemKind`,
   `Rarity` (в JSON — число), `WishTime` (проверка формата `ГГГГ-ММ-ДД чч:мм:сс`).
3. `domain::pity`: `PoolDefinition`/`PityRules`/`FiftyFiftyRule`/`CapturingRadianceRule`, `Probability` (сотые доли %),
   трейт `FeaturedLookup` + `NoFeaturedData`, `PityTracker` (счётчики, номера молитв по гаранту, 50/50, серия проигрышей,
   шанс следующего 5★ с баннера), `calculate_pool_statistics`, `annotate_wishes` (для таблицы на этапе 5).
4. `GameModule` дополнен методами `pools()` и `featured_lookup()`. `game-genshin`: коды 100/200/301/302/400/500, 5 пулов.
5. `application`: порт `WishRepository`, `StatisticsService` (+ `GameStatistics`, `PoolSummary`).
   `infrastructure`: `InMemoryWishRepository` — временное пустое хранилище до SQLite.
6. `app-desktop`: команда `get_game_statistics`, `AppState.game_catalog` теперь `Arc`.
7. Интерфейс: `GameSummary` (всего молитв и примогемов), новый `BannerCard` (молитвы, примогемы, до гаранта 5★/4★ со
   счётчиком), `FeaturedStatusView` (50/50, шанс, серия, правило Capturing Radiance); `i18n.t(key, params)`, `i18n.poolName`;
   `demo-data.ts` для `npm run dev` в браузере.
8. Vitest 5 подключён (`npm test`): тесты `format`, `interpolate`, `stats/featured`.

**Проверено:** 62 Rust-теста, 11 тестов интерфейса; clippy 0 предупреждений; svelte-check 0 ошибок; portable .exe собирается.
В окне приложения статистика пустая (история ещё не хранится), визуально проверялась только сборка.

### 2026-11-15 — Этап 3: хранилище SQLite
**Решение:** rusqlite (bundled) вместо sqlx, миграции написаны вручную (ADR-015).

**Сделано:**
1. `infrastructure::database`: `open_database` (WAL, synchronous=NORMAL, foreign_keys, busy_timeout),
   `open_in_memory_database` (для тестов), миграции через `PRAGMA user_version`, `DatabaseError::NewerSchema`.
2. Миграция `0001_create_wishes.sql`: таблица `wishes` с CHECK-ограничениями, UNIQUE (game_id, wish_id), индекс.
3. Порт `WishRepository` дополнен методом `add_wishes` (атомарно, дубликаты пропускаются, возвращает число новых).
4. `SqliteWishRepository` (Mutex<Connection>, подготовленные запросы, проверка строк через доменные типы).
   `InMemoryWishRepository` удалён: в тестах используется SQLite в памяти.
5. `AppPaths.database_file` = `data/wish-counter.sqlite`; `bootstrap.rs` открывает базу, `StartupError::Database`.
6. Сквозной тест `crates/app-desktop/tests/statistics_end_to_end.rs` (SQLite → StatisticsService → правила Genshin).

**Проверено:** 74 Rust-теста (73 модульных + 1 сквозной), 11 тестов UI; clippy 0; svelte-check 0; cargo doc 0.
.exe ≈ 6,6 МБ (+1,6 МБ SQLite), при запуске создаёт базу и применяет миграцию 1, основной процесс ≈ 32 МБ RAM.

### 2026-11-15 — Исправление portable-режима
- Найдено: WebView2 писал кэш в `%LOCALAPPDATA%\com.wishcounter.desktop\EBWebView`.
- Исправлено: окно с `"create": false` создаётся в `crates/app-desktop/src/window.rs` с `data_directory(data/webview)` (ADR-017).
  `AppPaths.webview_dir`. Старая папка в AppData удалена.
- Проверено запуском .exe: окно открывается, `data/webview/EBWebView` создаётся, в `%LOCALAPPDATA%`/`%APPDATA%` следов нет.
- Решение владельца: картинки и WebView2 в .exe не встраивать (ADR-018).

### 2026-11-15 — Этап 4a-1: справочник из данных Paimon.moe
**Решения владельца:** источник — Paimon.moe; сервер игрока выбирается вручную / читается из JSON Paimon.moe
(пересчёт времени как в Paimon.moe); картинки — кэш при первом показе + докачка при импорте (без кнопки «Скачать все»);
автообновление раз в сутки + кнопка; шанс нужного 4★ — в 4a-2. Этап 4 разбит на 4a-1, 4a-2, 4b, 4c, коммит на каждый шаг.

**Исследовано:** `banners.js` (5 разделов, у двойных баннеров `featured` уже содержит обоих персонажей),
`characters.js`, `weaponList.js`, `locales/items/ru.json` (ключ — английское имя). Время баннеров — по UTC+8;
флаги `timezoneDependent` (начало) и `timezoneDependentEnd` (конец) означают «пересчитать на пояс сервера»
(`src/functions/wish.js` Paimon.moe: `start - (8 - offset)`). Смещения серверов: Asia/China +8, America −5, Europe +1.

**Сделано:**
1. `domain::reference`: `LocalizedText`, `ItemInfo`, `ScheduleTime` (`asia_based`), `BannerInfo`, `ReferenceSnapshot`, `ReferenceData` (индекс).
2. `game_genshin::paimon`: `js_literal` (разборщик без выполнения кода, лимит вложенности 64), `converter`
   (предметы ≥3★, имена RU/EN, баннеры 5 пулов, выравнивание `-`/`_` в id, отчёт о неизвестных id).
3. `infrastructure::remote`: `HttpClient` (ureq/rustls, HTTPS, белый список хостов, без редиректов, лимиты), `GitHubRepository`.
4. `crates/tools/data-sync`: генерирует `crates/games/genshin/data/reference.json` (371 предмет, 224 баннера, ревизия 4b873a6).
5. `GameModule::embedded_reference`, `ReferenceService` (+ лог загрузки), команда `get_reference_summary`,
   блок «Справочные данные» в настройках с атрибуцией Paimon.moe (MIT); `THIRD_PARTY_NOTICES.md`.

**Проверено:** 112 Rust-тестов, 11 тестов UI; clippy 0; svelte-check 0. .exe 6,9 МБ (+0,2 МБ), справочник загружается < 1 мс,
RAM ≈ 33 МБ.

**Следующий шаг:** 4a-2 — сервер игрока, `FeaturedLookup` по расписанию баннеров, шанс нужного 4★.
