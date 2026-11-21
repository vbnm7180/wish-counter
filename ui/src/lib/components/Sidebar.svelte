<!--
  Боковое меню: главная, список игр, настройки.
  Список игр приходит из Rust-ядра, поэтому новая игра появится здесь
  автоматически после регистрации её модуля.
-->
<script lang="ts">
  import type { GameInfo } from '../api/types';
  import { i18n } from '../i18n/index.svelte';
  import { gameIconUrl } from '../images/item-image';
  import { navigation, type Page } from '../state/navigation.svelte';
  import IconImage from './IconImage.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Игры для меню. */
    games: GameInfo[];
  }

  let { games }: Props = $props();

  /** Размер иконки игры в меню (пиксели). */
  const GAME_ICON_SIZE = 24;

  const homePage: Page = { kind: 'home' };
  const settingsPage: Page = { kind: 'settings' };

  /** Создаёт описание страницы игры. */
  function gamePage(gameId: string): Page {
    return { kind: 'game', gameId };
  }

  /** Создаёт описание страницы импорта игры. */
  function importPage(gameId: string): Page {
    return { kind: 'import', gameId };
  }
</script>

<nav class="sidebar">
  <div class="logo">✦ {i18n.t('app.title')}</div>

  <button class="item" class:active={navigation.isActive(homePage)} onclick={() => navigation.goTo(homePage)}>
    {i18n.t('nav.home')}
  </button>

  <div class="section-title">{i18n.t('nav.games')}</div>
  {#each games as game (game.id)}
    <button
      class="item"
      class:active={navigation.isActive(gamePage(game.id))}
      onclick={() => navigation.goTo(gamePage(game.id))}
    >
      <IconImage src={gameIconUrl(game.id)} name={i18n.gameName(game.id)} size={GAME_ICON_SIZE} />
      <span>{i18n.gameName(game.id)}</span>
    </button>
    <button
      class="item sub-item"
      class:active={navigation.isActive(importPage(game.id))}
      onclick={() => navigation.goTo(importPage(game.id))}
    >
      {i18n.t('nav.import')}
    </button>
  {/each}

  <div class="spacer"></div>

  <button class="item" class:active={navigation.isActive(settingsPage)} onclick={() => navigation.goTo(settingsPage)}>
    ⚙ {i18n.t('nav.settings')}
  </button>
</nav>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    width: var(--sidebar-width);
    padding: var(--space-4);
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
  }

  .logo {
    padding: var(--space-4) var(--space-3) var(--space-6);
    font-size: 18px;
    font-weight: 700;
    color: var(--color-accent);
  }

  .section-title {
    padding: var(--space-4) var(--space-3) var(--space-1);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    text-align: left;
    background: transparent;
    border: none;
    border-radius: var(--radius-small);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  /* Вложенный пункт под игрой: отступ на ширину иконки игры. */
  .sub-item {
    padding-left: calc(var(--space-3) + 24px + var(--space-2));
    font-size: 14px;
  }

  .item:hover {
    background: var(--color-surface-hover);
  }

  .item.active {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .spacer {
    flex: 1;
  }
</style>
