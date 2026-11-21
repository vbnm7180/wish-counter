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
12. **Коммиты — только с подтверждения владельца.** После шага: проверки, список изменённых файлов,
   предложенное сообщение коммита — и ждать «да». Владелец просматривает изменения в IDE перед коммитом.

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

- Иконка Genshin — файл владельца `crates/games/genshin/assets/icon.png` (128×128, с плашкой «miHoYo»; вариант обрезки
  без плашки предложен владельцу — файл не менять без его решения).
- Личный файл экспорта Paimon.moe владельца: `target/paimon-export/paimon-moe-local-data.json` (не в git, не копировать
  в репозиторий). Проверка: `paimon_export_real.rs` (см. docs/development.md).
- Решения владельца по интерфейсу (2026-11-18, ADR-027): выбор сервера — выпадающий список + «i» с подсказкой на
  странице «Импорт» (этап 6), пустая история → сообщение «Импортируйте историю…»; выбор фона — этап 5 (после 4c),
  через `--app-background-image`. До этапа 6 `RegionSelector` остаётся на странице игры временно.
- Пожелания владельца (2026-11-18, ADR-029): фоны Genshin по умолчанию — ключевые арты версий (скачиваются по требованию);
  иконка игры в меню (этап 5); страница-справочник персонажей/оружия/версий — этап 11, после основного плана.
- 📌 Ближе к **этапу 10** напомнить владельцу обсудить формат дистрибутива (ZIP с одним .exe и т.п.).
- **Антивирус (ADR-036):** у владельца Касперский; исключения владелец добавлять не будет. Если сборка удалена
  (`os error 32`/`os error 5` при запуске тестов, файл исчез) — спросить у владельца вердикт и искать причину в коде.
  Не перебирать папки (`read_dir`) в рабочем коде без необходимости. Этап 7 (чтение кэша игры + сеть) — сначала прототип.
- Замечания владельца к таблице молитв — доработать в шаге 5d, после этапа 7 (вместе с артами баннеров).
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

### 2026-11-15 — Этап 4a-2: сервер игрока, исход 50/50 по расписанию, 4★ с баннера
**Сделано:**
1. `WishTime`: проверка существования даты (chrono), `to_date_time` / `from_date_time`.
2. `domain::region::ServerRegion` (america −5, europe +1, asia/china +8; `parse` без учёта регистра — для JSON Paimon.moe).
3. `domain::pity::schedule_lookup::ScheduleFeaturedLookup`: баннер пула, действовавший в момент молитвы
   (`asia_based` время сдвигается на `hours_behind_asia`, +1 мин к концу), 5★ → `featured`, 4★ → `featured_rare`.
4. `PityRules.four_star_featured`, `PoolStatistics.four_star_featured`, отдельное состояние 50/50 для 4★ в трекере;
   Genshin: персонажи 50%, оружие 75%.
5. `GameModule::featured_lookup` удалён (ADR-023). `StatisticsService` получает `ReferenceService` и `ProfileRepository`;
   `GameStatistics.server_region`.
6. Миграция 0002 `game_profiles`, `SqliteProfileRepository`, `ProfileService`, команда `set_server_region`,
   `SharedConnection` — одно соединение на все хранилища (ADR-024).
7. Интерфейс: `RegionSelector` (с подсказкой, пока сервер не выбран), `FourStarFeaturedView`, тексты «неизвестен»
   с/без выбранного сервера.
8. Сквозные тесты на **настоящем** справочнике: Дилюк на баннере Мити → проигрыш, Митя → гарант, the_flute → гарант 4★.

**Проверено:** 141 Rust-тест, 11 тестов UI; clippy 0; svelte-check 0. Существующая база v1 обновилась до v2 при запуске
(применена только миграция 2). RAM ≈ 32 МБ. Окно не проверялось вручную (нет истории для отображения до этапа 6).

### 2026-11-17 — Этап 4b: онлайн-обновление справочника
**Решение владельца:** коммиты — только после его подтверждения (правило 12).

**Сделано:**
1. `domain::reference`: `ReferenceSourceSpec`, `SourceFiles`, `ReferenceBuildError`, трейт `RemoteReference`;
   `GameModule::remote_reference` (по умолчанию `None`). Genshin: `paimon::remote::PaimonRemoteReference`, `sources_from_files`.
2. `application`: порты `ReferenceCache`, `ReferenceDownloader` (+ `DownloadError`); `ReferenceService` — `RwLock`,
   `new(catalog, cache)` выбирает более новый из встроенного/скачанного, `replace`; `ReferenceUpdater`
   (`UpdateMode::IfDue/Force`, `UpdateOutcome`, `UpdateError`, `ReferenceStatus` с `last_checked_at`, проверка данных).
3. `infrastructure`: `json_file` (общая атомарная запись, настройки переведены на неё), `JsonReferenceCache`
   (`data/catalog/<игра>.json` + `.state.json`, проверка имени файла), `GitHubReferenceDownloader`, `AppPaths.catalog_dir`.
4. `app-desktop`: `background.rs` (проверка при запуске в отдельном потоке), `events.rs` (`reference-updated`),
   команда `update_reference` (async + spawn_blocking), `get_reference_summary` возвращает `ReferenceStatus`.
5. `data-sync` использует `GitHubReferenceDownloader` + `PaimonRemoteReference` (один список файлов).
6. Интерфейс: кнопка «Обновить данные», время последней проверки, сообщения о результате (`reference/update-messages.ts` + тесты),
   `api/events.ts`; `GamePage` и `ReferenceInfo` перечитывают данные по событию.

**Проверено:** 166 Rust-тестов + 2 онлайн-теста (`--ignored`, с настоящим GitHub — прошли), 16 тестов UI; clippy 0; svelte-check 0.
Запуск .exe в пустой папке (`target/portable-check-4b`): фоновая проверка → `UpToDate` (ревизия Paimon.moe не менялась
с 15.11), записан `data/catalog/genshin.state.json`; второй запуск в тот же день в сеть не ходил.
Ветка «Updated» в живом приложении не наблюдалась (нет новой ревизии) — покрыта модульными тестами и онлайн-тестом сборки.

### 2026-11-18 — Замечания владельца по интерфейсу (план) и этап 4c: картинки
**Замечания (запланированы, ADR-027):** выбор сервера → выпадающий список + «i» с подсказкой на странице «Импорт» (этап 6),
пустая история → сообщение «Импортируйте историю…» (этап 6), общие компоненты Select/InfoTooltip и выбор фона (этап 5).
Заложено сейчас: CSS-переменная `--app-background-image` в `themes/base.css` (по умолчанию `none`).

**Сделано (4c):**
1. `RemoteReference::image_path` (по умолчанию `None`); Genshin: `static/images/{characters|weapons}/<id>.png`.
2. `application`: порты `ImageCache`, `ImageDownloader`; `ImageService` (кэш → справочник → скачивание из ревизии справочника,
   проверка PNG, запоминание неудач на сеанс). `ReferenceSummary.latest_banner.featured` (id + имена).
3. `infrastructure`: `json_file` → `files` (+ `read_bytes`, `write_bytes_atomically`, `is_safe_file_stem`),
   `FileImageCache`, `HttpClient::get_bytes` (≤ 2 МБ), `GitHubRepository::fetch_bytes`, `GitHubReferenceDownloader`
   реализует и `ImageDownloader`, `AppPaths.images_dir` = `data/cache/images`.
4. `app-desktop`: `image_protocol` (протокол `wcimg`, async, проверка пути), CSP `img-src` + `wcimg:`/`http://wcimg.localhost`.
5. Интерфейс: `images/item-image.ts` (+ тест), `ItemIcon.svelte` (lazy, рамка по редкости, заглушка), иконки персонажей
   последнего баннера в настройках, `i18n.localized`.

**Проверено:** 183 Rust-теста + 3 онлайн-теста (`--ignored`: справочник, обновление, картинка Ху Тао скачана и закэширована),
19 тестов UI; clippy 0; svelte-check 0. .exe 8,6 МБ (рост с 6,9 — HTTP/TLS из 4b), запуск в пустой папке
(`target/portable-check-4c`): создан `data/cache/images`, ошибок в логе нет.
**Не проверено вручную:** отображение картинок в окне (страница «Настройки» в окне не открывалась).

### 2026-11-18 — Этап 5a: общие компоненты, иконка игры, тесты компонентов
**Решения владельца:** иконка игры — своя (файл владельца); порядок: 5a → этап 6 (импорт) → 5b (таблица), 5c (фоны);
фоны Genshin по умолчанию — ключевые арты версий; страница-справочник — этап 11 (ADR-029, ADR-030).

**Сделано:**
1. Тесты компонентов: @testing-library/svelte 5 + jsdom 31; `vite.config.ts` (`test.environment`, `setupFiles`,
   `resolve.conditions: ['browser']` в тестах); `ui/src/test-setup.ts`.
2. Компоненты: `Select` (на `<select>`, обобщённый тип значения), `InfoTooltip` («i» + tooltip),
   `IconImage` (картинка с заглушкой-буквой; `ItemIcon` теперь обёртка над ним) — все с тестами.
3. Иконка игры: `GameModule::icon_png` (по умолчанию `None`), `ImageService::game_icon`; протокол `wcimg`:
   `/items/<игра>/<предмет>` и `/games/<игра>/icon` (`ImageRequest`); иконка в левом меню (24 px, заглушка «G»).

4. Иконка Genshin: владелец положил `crates/games/genshin/assets/icon.png` (128×128, 31 КБ), встроена через `include_bytes!`.

**Проверено:** 186 Rust-тестов, 26 тестов UI; clippy 0; svelte-check 0; .exe 8,6 МБ.
**Не проверено вручную:** вид иконки в меню окна.
`Select`/`InfoTooltip` пока нигде не используются — подключаются на странице «Импорт» (этап 6).

### 2026-11-19 — Этап 6a: страница «Импорт», импорт Paimon.moe
**Исследован файл владельца:** разделы `wish-counter-*` (character-event 301+400, weapon-event, standard, chronicled;
beginners отсутствует), у молитв нет id и редкости, `server: "Europe"`, `pity`/`rate` (1 — выиграно, 0 — проиграно, 2 — гарант).
Найден дефект: 5 молитв оружейного баннера (06.07.2026) записаны дважды.

**Сделано:**
1. `domain::import`: `HistoryParser`, `ParsedHistory`, `ImportFormatId`, `ImportError`; `GameModule::history_parsers`.
2. `game_genshin::paimon::backup::PaimonBackupParser`: редкость из справочника, искусственные id,
   `RepeatDetector` (пропуск повторно записанных кусков), сервер из файла.
3. `application`: `ImportService` (≤ 20 МБ, сохранение молитв и сервера, `ImportSummary`), `ImageService::prefetch`,
   `ProfileService::server_region`.
4. `app-desktop`: команды `import_history` (async), `get_server_region`; событие `history-updated`;
   `background::spawn_image_prefetch`.
5. Интерфейс: `ImportPage` (сервер — `Select` + `InfoTooltip`, файл Paimon.moe, отчёт), пункт «Импорт» в меню под игрой,
   пустое состояние `GamePage` с переходом на импорт; `RegionSelector` удалён; `import-messages.ts` + тесты.
6. `tests/paimon_export_real.rs` (`#[ignore]`): на файле владельца — 2467 молитв, гарант всех 4★/5★ и 50/50 всех 5★
   совпадают с Paimon.moe (кроме объяснимого: счётчик Paimon.moe завышен на 5 после повторов).

**Проверено:** 204 Rust-теста + сверка на реальном файле, 30 тестов UI; clippy 0; svelte-check 0; запуск .exe без ошибок.
**Не проверено вручную:** импорт через окно (выбор файла, отчёт, переход со страницы игры) — просить владельца.

**Правка по ревью владельца:** в карточке баннера подпись «счётчик N / 90» заменена отдельными строками
«Текущие молитвы 5★/4★» над «До гаранта 5★/4★» (то же оформление: текст слева, число справа).

### 2026-11-19 — Этап 6b: экспорт в Paimon.moe, UID игрока
**Решения владельца:** UIGF отложить до импорта из игры (этап 7); UID хранить в профиле; в UIGF экспортировать
только молитвы с настоящими id (ADR-033, ADR-034).

**Исследовано:** UIGF v4.2 (`UIGF-org/UIGF-org.github.io`: `item_id` — числовой id из игры, `id` молитвы — только
цифры ≤ 19, `uid` обязателен); словарь `api.uigf.org/dict/genshin/en.json` (MIT) покрывает весь справочник, кроме
вариантов Путешественника. Логика записи у Paimon.moe — `src/routes/wish/import.svelte` (rate = гарант + «с баннера»).

**Сделано:**
1. `ImportFormatId` → `HistoryFormatId`; `domain::import`: `HistoryExporter`, `ExportInput`, `ExportError`;
   `GameModule::history_exporters`.
2. `domain::profile`: `PlayerUid`, `GameProfile`. Миграция 0003 (UID, необязательный сервер), `ProfileRepository::uid/set_uid`,
   `ProfileService::profile/set_uid`; UID читается из `wish-uid` и сохраняется при импорте.
3. `game_genshin::paimon::backup_export::PaimonBackupExporter` (+ тесты, включая экспорт → импорт).
4. `ExportService`; команды `export_history` (диалог из Rust, `tauri-plugin-dialog`), `get_game_profile`
   (заменила `get_server_region`), `set_player_uid`; `commands/import.rs` → `commands/history.rs`.
5. Интерфейс: страница «Импорт и экспорт» (сервер, UID с «i», импорт, экспорт), сообщения экспорта (+ тесты).
6. Сверка на файле владельца: экспорт после импорта = исходный файл (кроме следствий 5 повторов на оружейном баннере).

**Проверено:** 221 Rust-тест + 2 сверки на реальном файле, 32 теста UI; clippy 0; svelte-check 0; .exe собирается;
на настоящей базе владельца (v2) при запуске применена только миграция 3.
**Не проверено вручную:** диалог сохранения и UID в окне; восстановление экспортированного файла на paimon.moe.

**Решение владельца:** ручной ввод (6c) отложен до окончания этапа 11; приоритет — импорт из Paimon.moe и из игры, экспорт.

### 2026-11-19 — Этап 5b: таблица молитв
**Сделано:**
1. `ScheduleFeaturedLookup::banner_name` — баннер, действовавший в момент молитвы.
2. `application::HistoryService` (`PoolHistory`, `WishRow`: время, предмет, название, тип, редкость, гарант, исход 50/50,
   баннер; новые сверху); команда `get_pool_history` (async).
3. Интерфейс: `WishHistoryPage` (кнопка «Посмотреть» на карточке, назад к игре), фильтры по редкости/типу/названию,
   по 50 строк на странице, `ItemIcon` в строках, цвета редкости; `lib/history/table.ts` + тесты; демо-данные.
4. Сверка на файле владельца (`real_history_rows_have_banners`): у всех 4★/5★ баннеров персонажа и оружия найден баннер.

**Проверено:** 226 Rust-тестов + 3 сверки на реальном файле, 39 тестов UI; clippy 0; svelte-check 0; .exe запускается без ошибок.
**Не проверено вручную:** вид таблицы в окне (картинки, фильтры, листание).

### 2026-11-19 — Этап 5c: свои фоны
**Решения владельца:** встроенных фонов нет — только свои картинки (ADR-037); арты баннеров — позже, для таблицы (5d);
замечания к таблице — после этапа 7; исключения в антивирус не добавлять (ADR-036).

**Исследовано:** ключевые арты версий на Fandom (сопоставление версии и арта через главное изображение страницы версии),
но `static.wikia.nocookie.net` недоступен из сети владельца; API лаунчера HoYoPlay доступен (только текущая версия).

**Сделано:**
1. `domain::background`: `BackgroundId` (проверка, `from_content` — FNV-1a), `ImageFormat` (по сигнатуре), `BackgroundImage`;
   `AppSettings.background` (старые settings.json совместимы).
2. `application`: порт `BackgroundStore`, `BackgroundService` (≤ 20 МБ, формат, без дублей, удаление сбрасывает выбор);
   `AppState.settings_service` теперь в `Arc`.
3. `infrastructure::backgrounds::FileBackgroundStore` (`data/backgrounds/<id>.<ext>` + `backgrounds.json`), `AppPaths.backgrounds_dir`.
4. `app-desktop`: команды `list_backgrounds`, `add_background` (диалог из Rust), `remove_background`; протокол `wcimg`:
   `/backgrounds/<id>` с MIME по формату.
5. Интерфейс: `BackgroundPicker` (+ тест), `lib/backgrounds/background.ts` (+ тест), блок «Фон» в настройках,
   полупрозрачное боковое меню; обводка заголовков и подзаголовков при фоне; карточки в настройках и таблице;
   цвет номера по гаранту как в Paimon.moe (линейная шкала, `HARD_PITY_5`: 90, оружие 80).
7. **Правки по ревью владельца (v2):** меню непрозрачное; заголовки обводкой `-webkit-text-stroke` (только при фоне, 6 px для заголовков, 4 px для подзаголовков);
   настройки в карточках (тема, язык, фон, справочные данные); таблица + фильтры в одной карточке; цвет номера по гаранту —
   линейная шкала как в Paimon.moe: `(hardPity - pity) / hardPity * 120` (55 → жёлтый, 76 → тёмно-оранжевый, 17 → зелёный);
   подзаголовок на главной — цвет темы + обводка при фоне.
6. **Касперский** удалил тестовую сборку `infrastructure` (`VHO:Trojan-PSW.Win32.Convagent.gen`), когда хранилище перебирало
   папку. Переделано на файл-список — срабатываний больше нет (ADR-036).

**Проверено:** 252 Rust-теста, 53 теста UI; clippy 0; svelte-check 0. Владелец проверил фон в окне; правки по ревью в окне не проверены.

**Следующий шаг:** этап 7 — импорт из игры (сначала прототип и проверка реакции антивируса).


### 2026-11-21 — Этап 7: импорт из игры, очистка данных

**Прототип проверен:** `crates/tools/game-import-proto` — находит data_2 через output_log.txt,
сканирует по сигнатуре URL, делает один запрос к API. С фальшивым ключом получил 403 (ожидаемо).
Касперский не среагировал ни при компиляции прототипа, ни при сборке релиза (ADR-036 — read_dir нет).

**Сделано:**
1. `domain::import`: трейт `GameApiParser` (`gacha_type_codes`, `parse_page`), `ApiWishPage`, `GameApiError`,
   `GachaTypeCodes`. `GameModule::game_api_parser` (по умолчанию `None`).
2. `domain::reference::ReferenceData::item_by_en_name` — поиск по английскому имени (для API, который возвращает имена).
3. `WishRepository::delete_wishes`, `ProfileRepository::delete_profile` — удаление данных для смены аккаунта.
4. `game_genshin::game_api::GenshinApiParser`: разбор JSON-страниц API, переворот порядка (API отдаёт новые сверху),
   все 6 баннеров, 7 тестов.
5. `application`: порт `GameApiClient` (GET-запрос для API игры, отдельный от GitHub-клиента);
   `GameApiImportService` (постраничный обход всех баннеров, пауза 250 мс между страницами,
   authkey только в памяти); `ClearGameService` (удаляет молитвы и профиль игры).
6. `infrastructure::remote::GameApiHttpClient`: реализация `GameApiClient` через ureq (только HTTPS,
   без перенаправлений, лимит 128 КБ / страница, без белого списка хостов — URL берётся из веб-кэша игры).
7. `app-desktop`: команды `import_from_game`, `clear_game_data`; поиск data_2 через output_log.txt,
   извлечение URL по сигнатуре, разбор параметров (authkey не логируется и не пишется на диск).
8. Интерфейс: карточки «Импорт из игры» и «Очистить данные» на странице «Импорт»; красная зона
   с подтверждением перед очисткой; сообщения об ошибках (`authkey_expired`, `game_import_failed`); словари ru/en.
9. Прототип `crates/tools/game-import-proto` остаётся в workspace для отладки.

10. **Исправления после проверки на игре (7.2, HoYoPlay):** в логе нет строки «MiHoYoSDKUtil: data path is» —
    папка игры берётся из любой строки с `GenshinImpact_Data` (от буквы диска); ссылка ищется по `getGachaLog`
    назад до `https://` (в кэше префикс «1/0/»), берётся последняя; хост API — из ссылки
    (у HoYoPlay-версии хост и путь отличаются от старых), только домены hoyoverse.com/mihoyo.com;
    параметры кодируются (в authkey есть «+», «/», «=»); язык ответа всегда `en-us` (поиск предметов по англ. имени);
    `game_biz` по умолчанию `hk4e_global`.
11. **Правка владельца:** дата в таблице — «19.11.2026 20:14» (`formatWishTime` + тест), у заголовка «Дата» — «i»
    с подсказкой «время сервера игры».
12. **Исправление (после коммита e6386a2):** после импорта из игры в таблице не было исхода 50/50 — импорт не сохранял
    сервер игрока, а без него расписание баннеров не сопоставить (после «Очистить данные» профиль пуст).
    `GameApiParser::server_region` (Genshin: `os_usa`/`os_euro`/`os_asia`/`os_cht`/`cn_gf01`/`cn_qd01`),
    `GameApiImportService` сохраняет сервер из параметра `region` ссылки в профиль, `GameApiImportSummary.server_region`;
    в отчёте импорта — «Сервер из игры: …»; после очистки страница сбрасывает сервер и UID. 260 Rust-тестов.

**Проверено:** 259 Rust-тестов, 56 UI-тестов; clippy 0; svelte-check 0; .exe собрался, Касперский не среагировал.
**Проверено владельцем:** импорт из игры на реальном аккаунте работает.

**Следующий шаг:** несколько аккаунтов с переключением (пожелание владельца); затем 5d (доработка таблицы + арты баннеров), UIGF.
