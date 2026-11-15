# Wish Counter

Портативное десктопное приложение для учёта молитв (круток) в гача-играх HoYoverse.
Первая поддерживаемая игра — **Genshin Impact**. Архитектура рассчитана на добавление
Honkai: Star Rail, Zenless Zone Zero и других игр.

> Статус: **этап 1 — каркас приложения** (окно, меню, темы, языки, настройки).
> План разработки — в [docs/roadmap.md](docs/roadmap.md).

## Возможности (текущие)

- окно приложения с боковым меню и страницей Genshin Impact (заготовка карточек баннеров);
- тёмная и светлая темы, есть механизм добавления новых тем;
- русский и английский интерфейс;
- portable-режим: все данные хранятся в папке `data/` рядом с `.exe`.

## Технологии

| Часть | Технология |
|---|---|
| Ядро (логика, данные) | Rust (edition 2024) |
| Окно приложения | Tauri 2 (использует встроенный в Windows WebView2) |
| Интерфейс | Svelte 5 + TypeScript, сборщик Vite |
| База данных | SQLite (с этапа 3) |

## Быстрый старт

Требования: Rust (через rustup), Node.js 20+, Visual Studio Build Tools (C++), Windows 10/11.
Подробная установка описана в [docs/development.md](docs/development.md).

```bash
npm install           # зависимости интерфейса (один раз)
npm run app:dev       # запуск в режиме разработки с автообновлением
npm run app:build     # portable-сборка: target/release/wish-counter.exe
cargo test --workspace  # тесты Rust
npm run check         # проверка типов интерфейса
```

## Документация

- [docs/architecture.md](docs/architecture.md) — архитектура, слои, структура папок;
- [docs/development.md](docs/development.md) — установка окружения, команды, как добавить тему, язык или игру;
- [docs/roadmap.md](docs/roadmap.md) — план разработки по этапам;
- [docs/decisions.md](docs/decisions.md) — журнал архитектурных решений;
- [docs/security.md](docs/security.md) — безопасность и соблюдение пользовательского соглашения игр;
- [docs/rust-notes.md](docs/rust-notes.md) — пояснения к конструкциям Rust, которые используются в проекте;
- [CLAUDE.md](CLAUDE.md) — инструкции и журнал работы для ИИ-агентов.

## Правовая информация

Неофициальный фан-проект, не связан с HoYoverse. Genshin Impact, Honkai: Star Rail,
Zenless Zone Zero и связанные с ними материалы принадлежат HoYoverse.
