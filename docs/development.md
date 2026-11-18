# Разработка

## Установка окружения (Windows)

1. **Visual Studio Build Tools** с компонентом «Разработка классических приложений на C++»:
   ```powershell
   winget install --id Microsoft.VisualStudio.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
   ```
2. **Rust.** На этом компьютере Rust установлен в `C:\Rust`, а не в профиль пользователя,
   потому что кириллица в пути профиля иногда ломает инструменты сборки:
   ```powershell
   setx RUSTUP_HOME C:\Rust\rustup
   setx CARGO_HOME C:\Rust\cargo
   # затем запустить rustup-init.exe с https://rustup.rs
   ```
   После установки нужно перезапустить терминал или VS Code, чтобы обновился `PATH`.
3. **Node.js** 20+ (`winget install OpenJS.NodeJS.LTS`).
4. **WebView2** уже встроен в Windows 10/11.
5. Расширения VS Code: rust-analyzer, Svelte, Tauri (VS Code сам предложит их по `.vscode/extensions.json`).

## Команды

| Команда | Что делает |
|---|---|
| `npm install` | Установить зависимости интерфейса |
| `npm run app:dev` | Запустить приложение в режиме разработки (изменения UI применяются на лету) |
| `npm run dev` | Только интерфейс в браузере (http://localhost:1420), ядро подменено заглушкой `BrowserBackend` |
| `npm run app:build` | Portable-сборка → `target/release/wish-counter.exe` |
| `npm run app:build:installer` | Сборка установщиков NSIS/MSI (задел на будущее) |
| `npm run check` | Проверка типов TypeScript/Svelte |
| `npm test` | Unit-тесты интерфейса (Vitest) |
| `npm run test:watch` | Тесты интерфейса с перезапуском при изменениях |
| `cargo test --workspace` | Все тесты Rust |
| `cargo clippy --workspace --all-targets` | Анализатор кода Rust |
| `cargo fmt --all` | Автоформатирование Rust-кода |
| `cargo run -p data-sync` | Обновить встроенный справочник из Paimon.moe (нужен интернет) |
| `cargo test -p wish-counter --test reference_online -- --ignored` | Проверить настоящее скачивание с GitHub (нужен интернет) |

## Версии Tauri

Версии npm-пакетов `@tauri-apps/api` и `@tauri-apps/cli` должны совпадать с крейтом `tauri`
по major.minor, иначе `tauri build` выдаст ошибку. Сейчас они закреплены на `~2.11`.
При обновлении крейта (`cargo update -p tauri`) нужно обновить и npm-пакеты.

## Как добавить…

### …обновить встроенный справочник (новый патч, новые персонажи)
Пользователям это не обязательно: приложение само скачивает обновления (раз в сутки или по кнопке
«Обновить данные» в настройках) в `data/catalog/`. Встроенный снимок обновляют перед выпуском новой версии.

1. `cargo run -p data-sync` — скачает последнюю ревизию Paimon.moe и перезапишет `crates/games/genshin/data/reference.json`.
2. Посмотреть вывод: число предметов и баннеров, предупреждения о неизвестных идентификаторах.
3. `cargo test -p game-genshin` — тесты проверят, что снимок корректен.
4. Закоммитить `reference.json` и пересобрать приложение.

### …миграцию базы данных
1. Создать `crates/infrastructure/src/database/migrations/sql/000N_описание.sql`.
2. Добавить `include_str!("sql/000N_описание.sql")` в конец `MIGRATIONS` (`migrations.rs`).
3. Уже выпущенные миграции **не менять**: у пользователей они уже применены.

### Как посмотреть базу
Файл `target/release/data/wish-counter.sqlite` (или `data/` рядом с dev-сборкой) открывается в
[DB Browser for SQLite](https://sqlitebrowser.org/) или через `sqlite3`. Перед этим лучше закрыть приложение.

### …тему оформления
1. Скопировать `ui/src/lib/themes/dark.css` в `<id>.css`, заменить селектор на `:root[data-theme='<id>']` и цвета.
2. Импортировать файл в `ui/src/main.ts`.
3. Добавить `{ id: '<id>', labelKey: 'theme.<id>' }` в `THEMES` (`themes/registry.ts`).
4. Добавить строку `'theme.<id>'` в `locales/ru.ts` и `locales/en.ts`.

### …язык
1. Создать `ui/src/lib/i18n/locales/<код>.ts` по образцу `en.ts`.
2. Добавить его в `DICTIONARIES` и `LANGUAGES` (`i18n/index.svelte.ts`) и в тип `LanguageCode` (`api/types.ts`).
3. Добавить вариант в `enum Language` (`crates/domain/src/settings.rs`) с `#[serde(rename = "<код>")]`.

### …игру
1. Создать крейт `crates/games/<игра>` по образцу `genshin`, реализовать трейт `GameModule`:
   `info()`, `pools()` (коды баннеров и правила гаранта, см. `genshin/src/pools/pools.rs`), `featured_lookup()`.
2. Добавить крейт в `members` и `[workspace.dependencies]` корневого `Cargo.toml`.
3. Зарегистрировать модуль в `build_state` (`crates/app-desktop/src/bootstrap.rs`).
4. Добавить строки `'games.<id>.name'` и `'<id>.banner.<pool_id>'` для каждого пула в словари.

## Стиль кода

- Имена — на английском, комментарии и документация — на русском.
- У каждой функции, структуры и модуля есть комментарий: что делает и зачем
  (`///` для элементов, `//!` для модулей). Lint `missing_docs` предупреждает о пропусках.
- Функции короткие и делают одно дело. Магические числа вынесены в именованные константы.
- `unsafe` запрещён на уровне workspace.
- Тесты — в отдельном файле рядом с кодом. Модуль с тестами `foo` — это папка `foo/` с `foo.rs` и `foo.tests.rs`
  (подключение через `#[path]`, пример — `crates/domain/src/lib.rs` и `crates/domain/src/settings/settings.rs`).
  Подробнее — ADR-009 в decisions.md.
- Стили компонента — в теге `<style>` внутри `.svelte` (Svelte изолирует их автоматически).
  Глобальные стили и цвета тем — только в `ui/src/lib/themes/`.
