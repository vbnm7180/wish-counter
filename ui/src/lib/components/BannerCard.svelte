<!--
  Карточка баннера: всего молитв, потраченные примогемы, текущие молитвы
  и сколько осталось до гарантов 5★ и 4★, статус 50/50, шанс 4★ с баннера и кнопка «Посмотреть» (таблица — этап 5).
-->
<script lang="ts">
  import type { PoolSummary } from '../api/types';
  import { formatNumber } from '../format/format';
  import { i18n } from '../i18n/index.svelte';
  import FeaturedStatusView from './FeaturedStatusView.svelte';
  import FourStarFeaturedView from './FourStarFeaturedView.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Название баннера (уже переведённое). */
    title: string;
    /** Статистика пула из ядра. */
    pool: PoolSummary;
    /** Выбран ли сервер игрока. */
    serverChosen: boolean;
    /** Вызывается по кнопке «Посмотреть». Если не задан — кнопка неактивна. */
    onView?: () => void;
  }

  let { title, pool, serverChosen, onView }: Props = $props();
</script>

<article class="card">
  <h3>{title}</h3>
  <dl>
    <dt>{i18n.t('game.total_pulls')}</dt>
    <dd>{formatNumber(pool.total_wishes, i18n.language)}</dd>

    <dt>{i18n.t('game.primogems')}</dt>
    <dd class="primogems">{formatNumber(pool.primogems_spent, i18n.language)}</dd>

    <dt>{i18n.t('game.current_5')}</dt>
    <dd class="rarity-5">{pool.five_star.current}</dd>
    <dt>{i18n.t('game.pity_5')}</dt>
    <dd class="rarity-5">{pool.five_star.remaining}</dd>

    <dt>{i18n.t('game.current_4')}</dt>
    <dd class="rarity-4">{pool.four_star.current}</dd>
    <dt>{i18n.t('game.pity_4')}</dt>
    <dd class="rarity-4">{pool.four_star.remaining}</dd>
  </dl>

  {#if pool.featured}
    <FeaturedStatusView status={pool.featured} {serverChosen} />
  {/if}

  {#if pool.four_star_featured}
    <FourStarFeaturedView status={pool.four_star_featured} />
  {/if}

  <button class="view" disabled={!onView} onclick={onView}>{i18n.t('game.view')}</button>
</article>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-large);
    box-shadow: var(--shadow-card);
  }

  h3 {
    font-size: 15px;
  }

  dl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-2) var(--space-4);
    align-items: center;
    margin: 0;
  }

  dt {
    color: var(--color-text-muted);
  }

  dd {
    margin: 0;
    font-weight: 600;
    text-align: right;
  }

  .primogems {
    color: var(--color-accent);
  }

  .rarity-5 {
    color: var(--color-rarity-5);
    font-size: 18px;
  }

  .rarity-4 {
    color: var(--color-rarity-4);
    font-size: 18px;
  }

  .view {
    margin-top: auto;
    padding: var(--space-2) var(--space-3);
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    border: none;
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .view:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
