// Конфигурация Vite — сборщика интерфейса.
// Исходники интерфейса лежат в ui/, результат сборки — в dist/
// (оттуда его забирает Tauri, см. crates/app-desktop/tauri.conf.json).
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

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
});
