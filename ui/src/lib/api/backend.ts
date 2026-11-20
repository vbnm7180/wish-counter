// Доступ к Rust-ядру приложения.
//
// Интерфейс Backend описывает, что умеет ядро. Есть две реализации:
// - TauriBackend — настоящая, вызывает команды Rust через invoke;
// - BrowserBackend — заглушка для запуска интерфейса в обычном браузере
//   (`npm run dev`), чтобы верстать без сборки Rust.
// Компоненты зависят только от интерфейса Backend (принцип D из SOLID).
import { invoke, isTauri } from '@tauri-apps/api/core';
import {
  demoGameStatistics,
  demoImportSummary,
  demoPoolHistory,
  demoReferenceSummary,
  demoServerRegion,
  setDemoServerRegion,
} from './demo-data';
import type {
  AppSettings,
  BackgroundImage,
  ExportResult,
  GameInfo,
  GameProfile,
  GameStatistics,
  ImportFormat,
  ImportSummary,
  PoolHistory,
  ReferenceSummary,
  ServerRegion,
  UpdateOutcome,
} from './types';

/** Операции, которые предоставляет Rust-ядро. */
export interface Backend {
  /** Список поддерживаемых игр. */
  listGames(): Promise<GameInfo[]>;
  /** Добавленные картинки фона. */
  listBackgrounds(): Promise<BackgroundImage[]>;
  /** Добавляет картинку фона: ядро покажет диалог выбора файла; null — пользователь отменил выбор. */
  addBackground(): Promise<BackgroundImage | null>;
  /** Удаляет картинку фона и возвращает настройки (выбор сбрасывается, если удалён выбранный фон). */
  removeBackground(id: string): Promise<AppSettings>;
  /** Текущие настройки. */
  getSettings(): Promise<AppSettings>;
  /** Сохраняет настройки и возвращает сохранённое значение. */
  updateSettings(settings: AppSettings): Promise<AppSettings>;
  /** Статистика молитв игры: итоги и карточки баннеров. */
  getGameStatistics(gameId: string): Promise<GameStatistics>;
  /** Сводка о справочнике игры (версия данных, количество предметов и баннеров). */
  getReferenceSummary(gameId: string): Promise<ReferenceSummary>;
  /** Профиль игрока в игре: сервер и UID. */
  getGameProfile(gameId: string): Promise<GameProfile>;
  /** Сохраняет UID игрока (проверяется в ядре). */
  setPlayerUid(gameId: string, uid: string): Promise<void>;
  /** История молитв пула баннеров для таблицы (новые сверху). */
  getPoolHistory(gameId: string, poolId: string): Promise<PoolHistory>;
  /** Экспортирует историю: ядро покажет диалог «Сохранить как». */
  exportHistory(gameId: string, format: ImportFormat): Promise<ExportResult>;
  /** Сохраняет сервер игрока для игры. */
  setServerRegion(gameId: string, region: ServerRegion): Promise<void>;
  /** Импортирует историю молитв из текста файла. */
  importHistory(gameId: string, format: ImportFormat, content: string): Promise<ImportSummary>;
  /** Проверяет и скачивает обновление справочника прямо сейчас. */
  updateReference(gameId: string): Promise<UpdateOutcome>;
}

/** Реализация через команды Tauri (см. crates/app-desktop/src/commands). */
class TauriBackend implements Backend {
  listGames(): Promise<GameInfo[]> {
    return invoke<GameInfo[]>('list_games');
  }

  listBackgrounds(): Promise<BackgroundImage[]> {
    return invoke<BackgroundImage[]>('list_backgrounds');
  }

  addBackground(): Promise<BackgroundImage | null> {
    return invoke<BackgroundImage | null>('add_background');
  }

  removeBackground(id: string): Promise<AppSettings> {
    return invoke<AppSettings>('remove_background', { id });
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

  getReferenceSummary(gameId: string): Promise<ReferenceSummary> {
    return invoke<ReferenceSummary>('get_reference_summary', { gameId });
  }

  getGameProfile(gameId: string): Promise<GameProfile> {
    return invoke<GameProfile>('get_game_profile', { gameId });
  }

  setPlayerUid(gameId: string, uid: string): Promise<void> {
    return invoke<void>('set_player_uid', { gameId, uid });
  }

  exportHistory(gameId: string, format: ImportFormat): Promise<ExportResult> {
    return invoke<ExportResult>('export_history', { gameId, format });
  }

  getPoolHistory(gameId: string, poolId: string): Promise<PoolHistory> {
    return invoke<PoolHistory>('get_pool_history', { gameId, poolId });
  }

  setServerRegion(gameId: string, region: ServerRegion): Promise<void> {
    return invoke<void>('set_server_region', { gameId, region });
  }

  importHistory(gameId: string, format: ImportFormat, content: string): Promise<ImportSummary> {
    return invoke<ImportSummary>('import_history', { gameId, format, content });
  }

  updateReference(gameId: string): Promise<UpdateOutcome> {
    return invoke<UpdateOutcome>('update_reference', { gameId });
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
    const defaults: AppSettings = { theme: 'dark', language: 'ru', background: null };
    return saved ? { ...defaults, ...(JSON.parse(saved) as Partial<AppSettings>) } : defaults;
  }

  async updateSettings(settings: AppSettings): Promise<AppSettings> {
    localStorage.setItem(BROWSER_SETTINGS_KEY, JSON.stringify(settings));
    return settings;
  }

  // В браузере нет диалога выбора файла и протокола картинок — фонов нет.
  async listBackgrounds(): Promise<BackgroundImage[]> {
    return [];
  }

  async addBackground(): Promise<BackgroundImage | null> {
    return null;
  }

  async removeBackground(_id: string): Promise<AppSettings> {
    return this.updateSettings({ ...(await this.getSettings()), background: null });
  }

  async getGameStatistics(gameId: string): Promise<GameStatistics> {
    return demoGameStatistics(gameId);
  }

  async getReferenceSummary(gameId: string): Promise<ReferenceSummary> {
    return demoReferenceSummary(gameId);
  }

  async getGameProfile(_gameId: string): Promise<GameProfile> {
    return { server_region: demoServerRegion(), uid: null };
  }

  async setPlayerUid(_gameId: string, _uid: string): Promise<void> {}

  async getPoolHistory(_gameId: string, poolId: string): Promise<PoolHistory> {
    return demoPoolHistory(poolId);
  }

  async exportHistory(_gameId: string, _format: ImportFormat): Promise<ExportResult> {
    return { path: 'C:/demo/paimon-moe-local-data.json', wish_count: 0 };
  }

  async setServerRegion(_gameId: string, region: ServerRegion): Promise<void> {
    setDemoServerRegion(region);
  }

  async importHistory(_gameId: string, _format: ImportFormat, content: string): Promise<ImportSummary> {
    return demoImportSummary(content);
  }

  async updateReference(_gameId: string): Promise<UpdateOutcome> {
    return { status: 'up_to_date' };
  }
}

/** Выбирает реализацию в зависимости от того, где запущен интерфейс. */
function createBackend(): Backend {
  return isTauri() ? new TauriBackend() : new BrowserBackend();
}

/** Единственный экземпляр доступа к ядру для всего интерфейса. */
export const backend: Backend = createBackend();
