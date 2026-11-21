<!--
  Страница игры: итоги (всего молитв, примогемы) и карточки баннеров.
  Если истории нет — предложение перейти на страницу «Импорт».
  Статистика приходит из Rust-ядра (команда get_game_statistics).
-->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import { onHistoryUpdated, onReferenceUpdated } from '../lib/api/events';
  import type { GameStatistics } from '../lib/api/types';
  import BannerCard from '../lib/components/BannerCard.svelte';
  import Card from '../lib/components/Card.svelte';
  import GameSummary from '../lib/components/GameSummary.svelte';
  import { i18n } from '../lib/i18n/index.svelte';
  import { navigation } from '../lib/state/navigation.svelte';

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

  // Справочник обновился в фоне или импортирована история — пересчитать статистику.
  // Функция, которую возвращает $effect, вызывается при уходе со страницы: отписка.
  $effect(() => {
    const reload = (updatedGameId: string) => {
      if (updatedGameId === gameId) void loadStatistics(gameId);
    };
    const subscriptions = [onReferenceUpdated(reload), onHistoryUpdated(reload)];
    return () => subscriptions.forEach((subscription) => void subscription.then((unsubscribe) => unsubscribe()));
  });
</script>

<section class="page" class:fit-content={statistics?.total_wishes === 0}>
  <Card isTopLevelCard={true}>
    <h1>{i18n.gameName(gameId)}</h1>

    {#if failed}
      <p class="error">{i18n.t('error.load_failed')}</p>
    {:else if statistics}
      {#if statistics.total_wishes === 0}
        <div class="empty">
          <p>{i18n.t('game.empty')}</p>
          <button class="go-import" onclick={() => navigation.goTo({ kind: 'import', gameId })}>
            {i18n.t('game.go_to_import')}
          </button>
        </div>
      {:else}
        <GameSummary totalWishes={statistics.total_wishes} primogemsSpent={statistics.primogems_spent} />

        <h2>{i18n.t('game.banners')}</h2>
        <div class="grid">
          {#each statistics.pools as pool (pool.pool_id)}
            <BannerCard
              title={i18n.poolName(gameId, pool.pool_id)}
              {pool}
              serverChosen={statistics.server_region !== null}
              onView={pool.total_wishes > 0
                ? () => navigation.goTo({ kind: 'history', gameId, poolId: pool.pool_id })
                : undefined}
            />
          {/each}
        </div>
      {/if}
    {/if}
  </Card>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .page.fit-content {
    width: fit-content;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    max-width: 720px;
    gap: var(--space-3);
    padding: var(--space-6);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-large);
  }

  .empty p {
    margin: 0;
  }

  .go-import {
    padding: var(--space-2) var(--space-4);
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    border: none;
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .error {
    margin: 0;
    color: var(--color-danger);
  }

  h1 {
    font-size: 26px;
  }

  h2 {
    font-size: 18px;
    margin-top: var(--space-2);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-4);
    align-items: stretch;
  }
</style>
