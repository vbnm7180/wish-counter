# Архитектура Wish Counter

## Общая идея

Приложение состоит из двух частей, работающих в одном процессе:

1. **Ядро на Rust**: данные, правила подсчёта, импорт, сеть, файлы.
2. **Интерфейс на Svelte**: отображение. Он работает внутри окна Tauri (WebView2)
   и обращается к ядру только через **команды Tauri** (`invoke('имя_команды')`).

Интерфейс не имеет прямого доступа к файлам, сети или ОС. Всё это делает ядро
(см. [security.md](security.md)).

## Слои ядра (Clean Architecture)

```
        ┌──────────────────────────────┐
        │  app-desktop (Tauri, main)   │  ← сборка всех частей, команды для UI
        └──────────────┬───────────────┘
           ┌───────────┼────────────────┐
           ▼           ▼                ▼
   infrastructure   application     games/genshin
   (файлы, SQLite,  (сценарии,      (правила
    сеть, логи)      порты-трейты)   конкретной игры)
           │           │                │
           └───────────┴───────┬────────┘
                               ▼
                            domain
               (сущности и правила, без зависимостей)
```

Правило зависимостей: стрелки направлены **только внутрь**, к `domain`.
`domain` ничего не знает о других слоях. `application` объявляет трейты (порты),
`infrastructure` их реализует, а `app-desktop` связывает реализацию с портом.

| Крейт | Папка | Ответственность |
|---|---|---|
| `domain` | `crates/domain` | Сущности: `GameId`, `GameModule`, `AppSettings`, `Wish`, `Rarity`, `ServerRegion`; справочник (`ReferenceData`); правила гаранта (`PoolDefinition`, `PityRules`), расчёт (`PityTracker`), исход 50/50 по расписанию (`ScheduleFeaturedLookup`) |
| `application` | `crates/application` | Порты (`SettingsRepository`, `WishRepository`, `ProfileRepository`, `ReferenceCache`, `ReferenceDownloader`, `ImageCache`, `ImageDownloader`) и сервисы (`GameCatalog`, `SettingsService`, `StatisticsService`, `ReferenceService`, `ReferenceUpdater`, `ImageService`, `ProfileService`) |
| `infrastructure` | `crates/infrastructure` | `AppPaths` (portable/профиль), `JsonSettingsRepository`, база SQLite (`open_database`, миграции), `SqliteWishRepository`, `init_logging` |
| `game-genshin` | `crates/games/genshin` | `GenshinModule`; коды баннеров и правила пулов (`pools.rs`) |
| `wish-counter` | `crates/app-desktop` | Точка входа, сборка зависимостей (`bootstrap.rs`), команды Tauri |

### Как SOLID проявляется в коде

- **S (единственная ответственность):** у каждого модуля одна задача. `paths.rs` отвечает
  только за пути, `logging.rs` только за логи, команды Tauri только передают вызов сервису.
- **O (открытость/закрытость):** новая игра — это новый крейт с реализацией `GameModule`
  и одна строка регистрации в `bootstrap.rs`. Новая тема — CSS-файл и запись в реестре.
- **L (подстановка Лисков):** любая реализация `SettingsRepository` (JSON, в памяти в тестах,
  позже SQLite) взаимозаменяема.
- **I (разделение интерфейсов):** трейты маленькие и узкие (`SettingsRepository` — 2 метода).
- **D (инверсия зависимостей):** сервисы получают зависимости через конструктор
  (`SettingsService::new(Arc<dyn SettingsRepository>)`), а конкретные классы
  создаются в одном месте, в `bootstrap.rs`.

## Структура папок

```
wish-counter/
├── Cargo.toml              # Cargo workspace: список крейтов, общие версии, lints, release-профиль
├── package.json            # зависимости и скрипты интерфейса
├── vite.config.ts          # сборщик интерфейса (ui/ → dist/)
├── svelte.config.js, tsconfig.json
├── assets/branding/        # исходник иконки (SVG)
├── crates/
│   ├── domain/src/
│   │   ├── lib.rs          # корень крейта, описание модулей
│   │   ├── game/
│   │   │   ├── game.rs         # GameId, GameInfo, трейт GameModule
│   │   │   └── game.tests.rs   # тесты game.rs (так устроены все модули с тестами)
│   │   ├── background/     # background.rs: BackgroundId (из содержимого, FNV-1a), ImageFormat (по сигнатуре), BackgroundImage + тесты
│   │   ├── settings/
│   │   │   ├── settings.rs     # AppSettings (тема, язык, фон), ThemeId (с проверкой), Language
│   │   │   └── settings.tests.rs
│   │   ├── wish/
│   │   │   ├── wish.rs         # Wish, WishId, ItemId, BannerCode, ItemKind, Rarity, WishTime
│   │   │   └── wish.tests.rs
│   │   ├── import/         # import.rs: HistoryParser, HistoryExporter, ParsedHistory, ExportInput, HistoryFormatId
│   │   ├── profile/        # profile.rs: PlayerUid (проверка), GameProfile (сервер + UID) + тесты
│   │   ├── region/         # region.rs: ServerRegion (смещения часовых поясов как в Paimon.moe) + тесты
│   │   ├── reference/      # reference.rs: ItemInfo, BannerInfo, ScheduleTime, LocalizedText, ReferenceSnapshot, ReferenceData + тесты
│   │   └── pity/
│   │       ├── mod.rs          # описание модуля гаранта
│   │       ├── rules.rs        # PoolId, Probability, PityRules, FiftyFiftyRule, CapturingRadianceRule, PoolDefinition
│   │       ├── featured.rs     # трейт FeaturedLookup («с баннера ли 5★»), заглушка NoFeaturedData
│   │       ├── statistics.rs   # PityCounter, FeaturedStatus, ChanceSource, WishAnnotation, PoolStatistics
│   │       ├── schedule_lookup/ # ScheduleFeaturedLookup: «с баннера ли предмет» по расписанию и серверу + тесты
│   │       └── tracker/        # tracker.rs: PityTracker, calculate_pool_statistics, annotate_wishes + тесты
│   ├── application/src/
│   │   ├── ports/          # трейты для внешнего мира (SettingsRepository, WishRepository, ProfileRepository,
│   │   │                   #   ReferenceCache, ReferenceDownloader, ImageCache, ImageDownloader, BackgroundStore,
│   │   │                   #   RepositoryError, DownloadError)
│   │   └── services/       # game_catalog/, settings_service/, statistics_service/, reference_service/,
│   │                       #   reference_updater/ (онлайн-обновление), image_service/ (картинки), import_service/ (импорт),
│   │                       #   export_service/ (экспорт), history_service/ (строки таблицы молитв),
│   │                       #   profile_service/, background_service/ (свои фоны) (+ тесты)
│   ├── infrastructure/src/
│   │   ├── paths/          # paths.rs: AppPaths, StorageMode (Portable / UserProfile) + paths.tests.rs
│   │   ├── logging.rs      # init_logging: файл + консоль, ротация логов
│   │   ├── settings/       # json_settings_repository/: JsonSettingsRepository (атомарная запись) + тесты
│   │   ├── database/       # mod.rs: DatabaseError; shared_connection.rs: SharedConnection (одно соединение на все хранилища)
│   │   │   ├── connection/ # connection.rs: open_database (WAL, настройки) + тесты
│   │   │   └── migrations/ # migrations.rs: apply_migrations (PRAGMA user_version) + тесты;
│   │   │                   #   sql/0001_create_wishes.sql, 0002_create_game_profiles.sql, 0003_profile_uid.sql
│   │   ├── files/          # read_bytes/read_json, write_*_atomically, is_safe_file_stem — общие для файловых хранилищ + тесты
│   │   ├── image_cache/    # file_image_cache/: FileImageCache (data/cache/images/<игра>/<предмет>.png) + тесты
│   │   ├── backgrounds/    # file_background_store/: FileBackgroundStore (data/backgrounds/<id>.<ext> + backgrounds.json,
│   │   │                   #   без перебора папки — ADR-036) + тесты
│   │   ├── reference_cache/ # json_reference_cache/: JsonReferenceCache (data/catalog/) + тесты
│   │   ├── remote/         # http_client/ (HTTPS, белый список, лимиты), github/ (ревизия, файлы),
│   │   │                   #   github_downloader.rs (GitHubReferenceDownloader: справочник и картинки) + тесты
│   │   ├── profiles/       # sqlite_profile_repository/: SqliteProfileRepository (сервер игрока) + тесты
│   │   └── wishes/         # sqlite_wish_repository/: SqliteWishRepository + тесты
│   ├── games/genshin/src/
│   │   ├── lib.rs          # GenshinModule
│   │   ├── lib.tests.rs    # тесты
│   │   ├── pools/          # pools.rs: коды баннеров (banner_codes), пулы и правила + pools.tests.rs
│   │   ├── paimon/         # mod.rs: репозиторий и пути файлов Paimon.moe
│   │   │   ├── js_literal/ # разборщик JS-литералов (без выполнения кода) + тесты
│   │   │   ├── backup/     # PaimonBackupParser: импорт paimon-moe-local-data.json (повторы, искусственные id) + тесты
│   │   │   ├── backup_export/ # PaimonBackupExporter: экспорт в тот же формат (pity, rate, rateoff — как у Paimon.moe) + тесты
│   │   │   ├── converter/  # Paimon.moe → ReferenceSnapshot + тесты
│   │   │   └── remote/     # PaimonRemoteReference: источник обновлений (RemoteReference) + тесты
│   │   └── reference/      # встроенный снимок (include_str!) + тесты
│   ├── games/genshin/data/reference.json   # снимок справочника (генерирует data-sync)
│   ├── tools/data-sync/    # инструмент разработчика: скачать Paimon.moe → reference.json
│   └── app-desktop/
│       ├── tauri.conf.json # окно, CSP, сборка, установщик
│       ├── capabilities/   # права интерфейса (минимальные)
│       ├── icons/          # сгенерированные иконки
│       ├── tests/          # statistics_end_to_end.rs; reference_online.rs (#[ignore], интернет);
│       │                   #   paimon_export_real.rs (#[ignore], сверка с Paimon.moe на личном файле)
│       └── src/
│           ├── main.rs     # точка входа
│           ├── bootstrap.rs# composition root: создание и связывание объектов
│           ├── window.rs   # создание главного окна (папка данных WebView2 → data/webview)
│           ├── background.rs # фоновая проверка обновлений справочников при запуске
│           ├── events.rs   # события ядро → интерфейс (reference-updated)
│           ├── image_protocol/ # протокол wcimg: картинки для <img> (разбор и проверка адреса) + тесты
│           ├── state.rs    # AppState: сервисы, доступные командам
│           ├── error.rs    # StartupError, CommandError (ошибка для UI)
│           └── commands/   # команды Tauri: games.rs (list_games, get_game_statistics, get_game_profile,
│                           #   set_server_region, set_player_uid),
│                           #   reference.rs (get_reference_summary, update_reference — async), settings.rs,
│                           #   history.rs (import_history, export_history, get_pool_history — async;
│                           #   диалог «Сохранить как» из Rust)
├── ui/
│   ├── index.html
│   └── src/
│       ├── main.ts         # точка входа интерфейса
│       ├── test-setup.ts   # общая настройка тестов (очистка страницы после каждого теста)
│       ├── App.svelte      # каркас: меню + текущая страница
│       ├── pages/          # HomePage, GamePage (пустое состояние → «Импорт»), ImportPage (профиль, импорт, экспорт),
│       │                   #   WishHistoryPage (таблица молитв), SettingsPage
│       └── lib/
│           ├── api/        # backend.ts (вызовы ядра), events.ts (события ядра), types.ts, demo-data.ts
│           ├── reference/  # update-messages.ts: тексты результата обновления + тест
│           ├── images/     # item-image.ts: адреса картинок и иконок игр (wcimg) + тест
│           ├── i18n/       # локализация: index.svelte.ts, interpolate.ts ({параметры}), locales/ru.ts, en.ts
│           ├── format/     # format.ts: числа и проценты по языку + format.test.ts
│           ├── stats/      # featured.ts: что показать в блоке 50/50 + featured.test.ts
│           ├── themes/     # registry.ts, base.css, dark.css, light.css
│           ├── state/      # реактивное состояние: settings, navigation
│           └── components/ # Sidebar, GameSummary, BannerCard, FeaturedStatusView, FourStarFeaturedView,
│                           #   ReferenceInfo, IconImage, ItemIcon, Select, InfoTooltip (+ *.test.ts)
│           ├── history/    # table.ts: фильтры и постраничный вывод таблицы молитв + тест
│           ├── import/     # import-messages.ts (тексты результата импорта) + тест; regions.ts (список серверов)
└── docs/                   # документация
```

## Данные во время работы

В portable-режиме рядом с `wish-counter.exe` создаётся папка `data/`. **Вне этой папки приложение ничего не пишет**
(проверено: в `%LOCALAPPDATA%` и `%APPDATA%` следов нет):

```
data/
├── settings.json         # тема и язык
├── wish-counter.sqlite   # база: история молитв (+ служебные -wal, -shm пока приложение запущено)
├── webview/              # служебные данные WebView2 (кэш движка окна)
├── catalog/              # genshin.json — скачанный справочник; genshin.state.json — время проверки
├── cache/images/<игра>/  # картинки предметов (<id>.png), скачиваются при первом показе
└── logs/                 # wish-counter.YYYY-MM-DD.log, хранятся последние 7 файлов
```


## Справочные данные

```
Paimon.moe (GitHub) ──data-sync──► crates/games/genshin/data/reference.json ──include_str!──► .exe
                                     (разработчик запускает и коммитит)
.exe ► GameModule::embedded_reference() ► ReferenceService (ReferenceData с индексом) ► команды / расчёт
```
Справочник разбирается при запуске один раз (< 1 мс); если в `data/catalog/` есть скачанный справочник новее
встроенного — используется он.

### Онлайн-обновление (этап 4b)

```
запуск ─► background.rs (отдельный поток) ─► ReferenceUpdater::update_all_due
кнопка ─► update_reference (async, spawn_blocking) ─► ReferenceUpdater::update(Force)

ReferenceUpdater: прошло ≥ 24 ч? ─► latest_revision (1 запрос) ─► та же? ─► UpToDate
                                                         └─ новая ─► download ─► RemoteReference::build
                                                            ─► проверка ─► ReferenceCache::save ─► ReferenceService::replace
                                                            ─► событие reference-updated ─► интерфейс перечитывает данные
```
Источник обновлений описывает игра (`GameModule::remote_reference` → `RemoteReference`), скачивание общее
(`GitHubReferenceDownloader`). При ошибке справочник не меняется, время проверки не записывается.

## Картинки предметов (этап 4c)

```
<img src="http://wcimg.localhost/items/genshin/hu_tao">    (ItemIcon → IconImage, loading="lazy")
  └► image_protocol::handle_request — проверка адреса (/items/<игра>/<предмет>, /games/<игра>/icon, /backgrounds/<фон>)
     └► spawn_blocking ─► ImageService::image
          ├─ есть в data/cache/images ─► отдать
          ├─ предмета нет в справочнике ─► 404 (произвольный путь скачать нельзя)
          └─ RemoteReference::image_path ─► GitHub (ревизия справочника) ─► проверка PNG ─► кэш ─► отдать
```
Неудача (нет сети, нет файла) запоминается до конца сеанса — интерфейс показывает заглушку с буквой.
Иконка игры (`/games/<игра>/icon`) не скачивается: её байты встроены в модуль игры (`GameModule::icon_png`).

## Свой фон (этап 5c)

```
Настройки → «Добавить» → invoke('add_background')
  └► spawn_blocking ─► диалог выбора файла (Rust, tauri-plugin-dialog) ─► размер ≤ 20 МБ (до чтения)
     ─► BackgroundService::add: формат по сигнатуре (PNG/JPEG/WebP), id = FNV-1a от содержимого
     ─► FileBackgroundStore: data/backgrounds/<id>.<ext> + запись в backgrounds.json
Выбор: update_settings({ background: id }) → settingsState.apply → applyBackground:
  --app-background-image: url("http://wcimg.localhost/backgrounds/<id>") + <html data-background="on">
  (картинка показывается как есть; полупрозрачно только меню — --color-sidebar; текст на фоне — на плашках .plate)
Удаление: remove_background → файл и запись удалены; если фон был выбран — AppSettings.background = null
```

## Импорт истории (этап 6a)

```
ImportPage: <input type="file"> → file.text() → invoke('import_history', { gameId, format, content })
  └► spawn_blocking ─► ImportService::import
        ├─ GameModule::history_parsers() → PaimonBackupParser::parse (редкость — из справочника)
        ├─ WishRepository::add_wishes (уже сохранённые пропускаются)
        └─ ProfileRepository::set_server_region (если сервер есть в файле)
  └► событие history-updated → GamePage перечитывает статистику
  └► background::spawn_image_prefetch → ImageService::prefetch (картинки для офлайна)
```

## Экспорт истории (этап 6b)

```
ImportPage «Сохранить файл…» → invoke('export_history', { gameId, format })
  └► spawn_blocking ─► ExportService::export → GameModule::history_exporters() → PaimonBackupExporter
  └► spawn_blocking ─► tauri-plugin-dialog: blocking_save_file (диалог из Rust) → std::fs::write
```

## Таблица молитв (этап 5b)

```
BannerCard «Посмотреть» → WishHistoryPage → invoke('get_pool_history', { gameId, poolId })
  └► HistoryService::pool_history: молитвы пула → annotate_wishes (гарант, 50/50)
     + ScheduleFeaturedLookup::banner_name (баннер в момент молитвы) + названия из справочника
     → строки от новых к старым
  └► интерфейс: filterRows (редкость, тип, поиск) → pageRows (по 50) → строки с ItemIcon (lazy)
```

## Поток данных: статистика молитв

1. `GamePage.svelte` → `backend.getGameStatistics('genshin')` → `invoke('get_game_statistics', { gameId })`
2. `commands::games::get_game_statistics` → `StatisticsService::game_statistics`
3. Сервис берёт модуль игры из `GameCatalog`, молитвы — из `WishRepository` (SQLite, `ORDER BY wish_time, row_id`),
   сервер игрока — из `ProfileRepository`, справочник — из `ReferenceService`. Если сервер выбран, строится
   `ScheduleFeaturedLookup` (исход 50/50 по расписанию баннеров), иначе `NoFeaturedData` (статус «неизвестен»)
4. Для каждого пула (`GameModule::pools`) молитвы фильтруются по кодам баннеров и передаются в
   `calculate_pool_statistics`. `PityTracker` проходит по ним и считает счётчики и статус 50/50
5. Сервис добавляет примогемы (`PoolDefinition::primogems_for`) и итоги по игре → JSON → карточки `BannerCard`

## Поток данных на примере смены темы

1. `SettingsPage.svelte` → `settingsState.update({ theme })`
2. Тема сразу применяется к странице (`applyTheme`, атрибут `data-theme`)
3. `backend.updateSettings()` → `invoke('update_settings', { settings })`
4. Tauri проверяет JSON: `ThemeId` отклонит недопустимую строку
5. `commands::settings::update_settings` → `SettingsService::update_settings`
6. `JsonSettingsRepository::save` → атомарная запись в `data/settings.json`
7. Если сохранение не удалось, интерфейс откатывает тему и показывает ошибку
