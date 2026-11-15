// Доступ к Rust-ядру приложения.
//
// Интерфейс Backend описывает, что умеет ядро. Есть две реализации:
// - TauriBackend — настоящая, вызывает команды Rust через invoke;
// - BrowserBackend — заглушка для запуска интерфейса в обычном браузере
//   (`npm run dev`), чтобы верстать без сборки Rust.
// Компоненты зависят только от интерфейса Backend (принцип D из SOLID).
import { invoke, isTauri } from '@tauri-apps/api/core';
import { demoGameStatistics } from './demo-data';
import type { AppSettings, GameInfo, GameStatistics } from './types';

/** Операции, которые предоставляет Rust-ядро. */
export interface Backend {
  /** Список поддерживаемых игр. */
  listGames(): Promise<GameInfo[]>;
  /** Текущие настройки. */
  getSettings(): Promise<AppSettings>;
  /** Сохраняет настройки и возвращает сохранённое значение. */
  updateSettings(settings: AppSettings): Promise<AppSettings>;
  /** Статистика молитв игры: итоги и карточки баннеров. */
  getGameStatistics(gameId: string): Promise<GameStatistics>;
}

/** Реализация через команды Tauri (см. crates/app-desktop/src/commands). */
class TauriBackend implements Backend {
  listGames(): Promise<GameInfo[]> {
    return invoke<GameInfo[]>('list_games');
  }

  getSettings(): Promise<AppSettings> {
    return invoke<AppSettings>('get_settings');
  }

  updateSettings(settings: AppSettings): Promise<AppSettings> {
    return invoke<AppSettings>('update_settings', { settings });
  }

  getGameStatistics(gameId: string): Promise<GameStatistics> {
    return invoke<GameStatistics>('get_game_statistics', { gameId });
  }
}

/** Ключ localStorage для настроек в режиме браузера. */
const BROWSER_SETTINGS_KEY = 'wish-counter.settings';

/** Заглушка для браузера: хранит настройки в localStorage, статистика — демо-данные. */
class BrowserBackend implements Backend {
  async listGames(): Promise<GameInfo[]> {
    return [{ id: 'genshin', display_order: 10 }];
  }

  async getSettings(): Promise<AppSettings> {
    const saved = localStorage.getItem(BROWSER_SETTINGS_KEY);
    return saved ? (JSON.parse(saved) as AppSettings) : { theme: 'dark', language: 'ru' };
  }

  async updateSettings(settings: AppSettings): Promise<AppSettings> {
    localStorage.setItem(BROWSER_SETTINGS_KEY, JSON.stringify(settings));
    return settings;
  }

  async getGameStatistics(gameId: string): Promise<GameStatistics> {
    return demoGameStatistics(gameId);
  }
}

/** Выбирает реализацию в зависимости от того, где запущен интерфейс. */
function createBackend(): Backend {
  return isTauri() ? new TauriBackend() : new BrowserBackend();
}

/** Единственный экземпляр доступа к ядру для всего интерфейса. */
export const backend: Backend = createBackend();
