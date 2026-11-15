<!--
  Страница игры: итоги (всего молитв, примогемы) и карточки баннеров.
  Статистика приходит из Rust-ядра (команда get_game_statistics).
-->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import type { GameStatistics } from '../lib/api/types';
  import BannerCard from '../lib/components/BannerCard.svelte';
  import GameSummary from '../lib/components/GameSummary.svelte';
  import { i18n } from '../lib/i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Идентификатор игры, например 'genshin'. */
    gameId: string;
  }

  let { gameId }: Props = $props();

  /** Статистика игры; null — ещё загружается. */
  let statistics = $state<GameStatistics | null>(null);
  /** Загрузка завершилась ошибкой. */
  let failed = $state(false);

  /** Загружает статистику игры из ядра. */
  async function loadStatistics(id: string): Promise<void> {
    failed = false;
    try {
      statistics = await backend.getGameStatistics(id);
    } catch (cause) {
      console.error('Не удалось загрузить статистику', cause);
      failed = true;
    }
  }

  // $effect перезапускается при смене gameId (переход на страницу другой игры).
  $effect(() => {
    void loadStatistics(gameId);
  });
</script>

<section class="page">
  <h1>{i18n.gameName(gameId)}</h1>

  {#if failed}
    <p class="error">{i18n.t('error.load_failed')}</p>
  {:else if statistics}
    <GameSummary totalWishes={statistics.total_wishes} primogemsSpent={statistics.primogems_spent} />
    {#if statistics.total_wishes === 0}
      <p class="hint">{i18n.t('game.placeholder')}</p>
    {/if}

    <h2>{i18n.t('game.banners')}</h2>
    <div class="grid">
      {#each statistics.pools as pool (pool.pool_id)}
        <BannerCard title={i18n.poolName(gameId, pool.pool_id)} {pool} />
      {/each}
    </div>
  {/if}
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    color: var(--color-danger);
  }

  h2 {
    font-size: 16px;
    margin-top: var(--space-2);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-4);
    align-items: stretch;
  }
</style>
