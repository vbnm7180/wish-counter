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
| `application` | `crates/application` | Порты (`SettingsRepository`, `WishRepository`, `ProfileRepository`) и сервисы (`GameCatalog`, `SettingsService`, `StatisticsService`, `ReferenceService`, `ProfileService`) |
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
│   │   ├── settings/
│   │   │   ├── settings.rs     # AppSettings, ThemeId (с проверкой), Language
│   │   │   └── settings.tests.rs
│   │   ├── wish/
│   │   │   ├── wish.rs         # Wish, WishId, ItemId, BannerCode, ItemKind, Rarity, WishTime
│   │   │   └── wish.tests.rs
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
│   │   ├── ports/          # трейты для внешнего мира (SettingsRepository, WishRepository, ProfileRepository, RepositoryError)
│   │   └── services/       # game_catalog/, settings_service/, statistics_service/, reference_service/, profile_service/ (+ тесты)
│   ├── infrastructure/src/
│   │   ├── paths/          # paths.rs: AppPaths, StorageMode (Portable / UserProfile) + paths.tests.rs
│   │   ├── logging.rs      # init_logging: файл + консоль, ротация логов
│   │   ├── settings/       # json_settings_repository/: JsonSettingsRepository (атомарная запись) + тесты
│   │   ├── database/       # mod.rs: DatabaseError; shared_connection.rs: SharedConnection (одно соединение на все хранилища)
│   │   │   ├── connection/ # connection.rs: open_database (WAL, настройки) + тесты
│   │   │   └── migrations/ # migrations.rs: apply_migrations (PRAGMA user_version) + тесты; sql/0001_create_wishes.sql, 0002_create_game_profiles.sql
│   │   ├── remote/         # http_client/ (HTTPS, белый список хостов, лимиты), github/ (ревизия, файлы) + тесты
│   │   ├── profiles/       # sqlite_profile_repository/: SqliteProfileRepository (сервер игрока) + тесты
│   │   └── wishes/         # sqlite_wish_repository/: SqliteWishRepository + тесты
│   ├── games/genshin/src/
│   │   ├── lib.rs          # GenshinModule
│   │   ├── lib.tests.rs    # тесты
│   │   ├── pools/          # pools.rs: коды баннеров (banner_codes), пулы и правила + pools.tests.rs
│   │   ├── paimon/         # mod.rs: репозиторий и пути файлов Paimon.moe
│   │   │   ├── js_literal/ # разборщик JS-литералов (без выполнения кода) + тесты
│   │   │   └── converter/  # Paimon.moe → ReferenceSnapshot + тесты
│   │   └── reference/      # встроенный снимок (include_str!) + тесты
│   ├── games/genshin/data/reference.json   # снимок справочника (генерирует data-sync)
│   ├── tools/data-sync/    # инструмент разработчика: скачать Paimon.moe → reference.json
│   └── app-desktop/
│       ├── tauri.conf.json # окно, CSP, сборка, установщик
│       ├── capabilities/   # права интерфейса (минимальные)
│       ├── icons/          # сгенерированные иконки
│       ├── tests/          # сквозные тесты: statistics_end_to_end.rs (SQLite → статистика)
│       └── src/
│           ├── main.rs     # точка входа
│           ├── bootstrap.rs# composition root: создание и связывание объектов
│           ├── window.rs   # создание главного окна (папка данных WebView2 → data/webview)
│           ├── state.rs    # AppState: сервисы, доступные командам
│           ├── error.rs    # StartupError, CommandError (ошибка для UI)
│           └── commands/   # команды Tauri: games.rs (list_games, get_game_statistics, set_server_region), reference.rs, settings.rs
├── ui/
│   ├── index.html
│   └── src/
│       ├── main.ts         # точка входа интерфейса
│       ├── App.svelte      # каркас: меню + текущая страница
│       ├── pages/          # HomePage, GamePage, SettingsPage
│       └── lib/
│           ├── api/        # backend.ts (вызовы ядра), types.ts (типы ядра), demo-data.ts (данные для браузера)
│           ├── i18n/       # локализация: index.svelte.ts, interpolate.ts ({параметры}), locales/ru.ts, en.ts
│           ├── format/     # format.ts: числа и проценты по языку + format.test.ts
│           ├── stats/      # featured.ts: что показать в блоке 50/50 + featured.test.ts
│           ├── themes/     # registry.ts, base.css, dark.css, light.css
│           ├── state/      # реактивное состояние: settings, navigation
│           └── components/ # Sidebar, RegionSelector, GameSummary, BannerCard, FeaturedStatusView, FourStarFeaturedView, ReferenceInfo
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
└── logs/                 # wish-counter.YYYY-MM-DD.log, хранятся последние 7 файлов
```

С этапа 4 там же появится кэш справочников и картинок.

## Справочные данные

```
Paimon.moe (GitHub) ──data-sync──► crates/games/genshin/data/reference.json ──include_str!──► .exe
                                     (разработчик запускает и коммитит)
.exe ► GameModule::embedded_reference() ► ReferenceService (ReferenceData с индексом) ► команды / расчёт
```
Справочник разбирается при запуске один раз (< 1 мс). На этапе 4b добавится скачанное обновление в `data/catalog/`.

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
