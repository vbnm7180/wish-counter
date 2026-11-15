// Конфигурация Svelte: включаем поддержку TypeScript в .svelte-файлах.
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
};
