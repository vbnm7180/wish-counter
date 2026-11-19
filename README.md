# Wish Counter

Портативное десктопное приложение для учёта молитв (круток) в гача-играх HoYoverse.
Первая поддерживаемая игра — **Genshin Impact**. Архитектура рассчитана на добавление
Honkai: Star Rail, Zenless Zone Zero и других игр.

> Статус: готовы каркас, расчёт гаранта, SQLite, справочник с онлайн-обновлением и картинками,
> импорт истории из Paimon.moe. Подробности — в плане.
> План разработки — в [docs/roadmap.md](docs/roadmap.md).

## Возможности (текущие)

- страница Genshin Impact: всего молитв и примогемов, карточки баннеров со счётчиками гаранта 5★/4★,
  статусом 50/50, шансом получить персонажа с баннера и Capturing Radiance;
- таблица молитв каждого баннера (кнопка «Посмотреть»): картинки, номер по гаранту, исход 50/50,
  баннер; фильтры по редкости, типу и названию;
- импорт и экспорт истории в формате резервной копии Paimon.moe (страница «Импорт и экспорт»),
  хранение в локальной базе SQLite;
- встроенный справочник: 128 персонажей, 243 оружия, 224 баннера (данные Paimon.moe), работает офлайн;
- тёмная и светлая темы, есть механизм добавления новых тем;
- русский и английский интерфейс;
- portable-режим: все данные хранятся в папке `data/` рядом с `.exe`.

## Технологии

| Часть | Технология |
|---|---|
| Ядро (логика, данные) | Rust (edition 2024) |
| Окно приложения | Tauri 2 (использует встроенный в Windows WebView2) |
| Интерфейс | Svelte 5 + TypeScript, сборщик Vite |
| База данных | SQLite (rusqlite, встроена в .exe) |

## Быстрый старт

Требования: Rust (через rustup), Node.js 20+, Visual Studio Build Tools (C++), Windows 10/11.
Подробная установка описана в [docs/development.md](docs/development.md).

```bash
npm install           # зависимости интерфейса (один раз)
npm run app:dev       # запуск в режиме разработки с автообновлением
npm run app:build     # portable-сборка: target/release/wish-counter.exe
cargo test --workspace  # тесты Rust
npm test              # тесты интерфейса (Vitest)
npm run check         # проверка типов интерфейса
```

## Документация

- [docs/architecture.md](docs/architecture.md) — архитектура, слои, структура папок;
- [docs/development.md](docs/development.md) — установка окружения, команды, как добавить тему, язык или игру;
- [docs/roadmap.md](docs/roadmap.md) — план разработки по этапам;
- [docs/decisions.md](docs/decisions.md) — журнал архитектурных решений;
- [docs/genshin-rules.md](docs/genshin-rules.md) — правила гаранта Genshin и решения по ним;
- [docs/security.md](docs/security.md) — безопасность и соблюдение пользовательского соглашения игр;
- [docs/rust-notes.md](docs/rust-notes.md) — пояснения к конструкциям Rust, которые используются в проекте;
- [CLAUDE.md](CLAUDE.md) — инструкции и журнал работы для ИИ-агентов;
- [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) — сторонние данные и их лицензии.

## Правовая информация

Неофициальный фан-проект, не связан с HoYoverse. Genshin Impact, Honkai: Star Rail,
Zenless Zone Zero и связанные с ними материалы принадлежат HoYoverse.
