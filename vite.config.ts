/// <reference types="vitest/config" />
// Конфигурация Vite — сборщика интерфейса — и Vitest — тестов интерфейса.
// Исходники интерфейса лежат в ui/, результат сборки — в dist/
// (оттуда его забирает Tauri, см. crates/app-desktop/tauri.conf.json).
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

/** Vitest выставляет эту переменную окружения при запуске тестов. */
const isTest = Boolean(process.env.VITEST);

export default defineConfig({
  root: 'ui',
  plugins: [svelte()],
  // Tauri ожидает dev-сервер на фиксированном порту.
  server: { port: 1420, strictPort: true },
  // Не очищать консоль, чтобы были видны сообщения Rust.
  clearScreen: false,
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    // WebView2 основан на свежем Chromium — можно не транспилировать под старые браузеры.
    target: 'chrome120',
  },
  // В тестах компоненты Svelte должны собираться в «браузерном» варианте,
  // иначе они собираются для сервера и не умеют работать с DOM.
  resolve: isTest ? { conditions: ['browser'] } : undefined,
  test: {
    // jsdom — имитация браузера в Node.js: в нём тесты рисуют компоненты.
    environment: 'jsdom',
    // Общая настройка: очистка страницы после каждого теста.
    setupFiles: ['src/test-setup.ts'],
  },
});
