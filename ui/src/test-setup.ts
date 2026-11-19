// Общая настройка тестов интерфейса (подключается в vite.config.ts → test.setupFiles).
//
// После каждого теста убираем нарисованные компоненты со страницы jsdom,
// иначе следующий тест увидит элементы предыдущего.
import { cleanup } from '@testing-library/svelte';
import { afterEach } from 'vitest';

afterEach(() => cleanup());
