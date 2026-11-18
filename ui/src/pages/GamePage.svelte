<!--
  Страница игры: сервер игрока, итоги (всего молитв, примогемы) и карточки баннеров.
  Статистика приходит из Rust-ядра (команда get_game_statistics).
-->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import { onReferenceUpdated } from '../lib/api/events';
  import type { GameStatistics, ServerRegion } from '../lib/api/types';
  import BannerCard from '../lib/components/BannerCard.svelte';
  import GameSummary from '../lib/components/GameSummary.svelte';
  import RegionSelector from '../lib/components/RegionSelector.svelte';
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
  /** Не удалось сохранить сервер. */
  let regionSaveFailed = $state(false);

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

  /** Сохраняет выбранный сервер и пересчитывает статистику. */
  async function selectRegion(region: ServerRegion): Promise<void> {
    regionSaveFailed = false;
    try {
      await backend.setServerRegion(gameId, region);
      await loadStatistics(gameId);
    } catch (cause) {
      console.error('Не удалось сохранить сервер', cause);
      regionSaveFailed = true;
    }
  }

  // $effect перезапускается при смене gameId (переход на страницу другой игры).
  $effect(() => {
    void loadStatistics(gameId);
  });

  // Справочник обновился в фоне — исход 50/50 мог измениться, пересчитать статистику.
  // Функция, которую возвращает $effect, вызывается при уходе со страницы: отписка.
  $effect(() => {
    const subscription = onReferenceUpdated((updatedGameId) => {
      if (updatedGameId === gameId) void loadStatistics(gameId);
    });
    return () => void subscription.then((unsubscribe) => unsubscribe());
  });
</script>

<section class="page">
  <h1>{i18n.gameName(gameId)}</h1>

  {#if failed}
    <p class="error">{i18n.t('error.load_failed')}</p>
  {:else if statistics}
    <RegionSelector selected={statistics.server_region} onSelect={(region) => void selectRegion(region)} />
    {#if regionSaveFailed}
      <p class="error">{i18n.t('region.save_error')}</p>
    {/if}

    <GameSummary totalWishes={statistics.total_wishes} primogemsSpent={statistics.primogems_spent} />
    {#if statistics.total_wishes === 0}
      <p class="hint">{i18n.t('game.placeholder')}</p>
    {/if}

    <h2>{i18n.t('game.banners')}</h2>
    <div class="grid">
      {#each statistics.pools as pool (pool.pool_id)}
        <BannerCard
          title={i18n.poolName(gameId, pool.pool_id)}
          {pool}
          serverChosen={statistics.server_region !== null}
        />
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
