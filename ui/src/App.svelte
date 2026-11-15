<!--
  Корневой компонент: при запуске загружает настройки и список игр,
  рисует боковое меню и текущую страницу.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { backend } from './lib/api/backend';
  import type { GameInfo } from './lib/api/types';
  import Sidebar from './lib/components/Sidebar.svelte';
  import { i18n } from './lib/i18n/index.svelte';
  import { navigation } from './lib/state/navigation.svelte';
  import { settingsState } from './lib/state/settings.svelte';
  import GamePage from './pages/GamePage.svelte';
  import HomePage from './pages/HomePage.svelte';
  import SettingsPage from './pages/SettingsPage.svelte';

  /** Список игр из ядра. */
  let games = $state<GameInfo[]>([]);
  /** Не удалось загрузить список игр. */
  let gamesFailed = $state(false);

  /** Загружает список игр из Rust-ядра. */
  async function loadGames(): Promise<void> {
    try {
      games = await backend.listGames();
    } catch (cause) {
      console.error('Не удалось загрузить список игр', cause);
      gamesFailed = true;
    }
  }

  onMount(() => {
    void settingsState.load();
    void loadGames();
  });
</script>

<div class="layout">
  <Sidebar {games} />
  <main class="content">
    {#if gamesFailed}
      <p class="error">{i18n.t('error.load_failed')}</p>
    {/if}

    {#if navigation.current.kind === 'home'}
      <HomePage />
    {:else if navigation.current.kind === 'game'}
      <GamePage gameId={navigation.current.gameId} />
    {:else if navigation.current.kind === 'settings'}
      <SettingsPage {games} />
    {/if}
  </main>
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .layout {
    display: flex;
    height: 100%;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-8);
  }

  .error {
    color: var(--color-danger);
  }
</style>
