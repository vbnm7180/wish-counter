<!--
  Карточка баннера: всего молитв, потраченные примогемы, сколько осталось
  до гарантов 5★ и 4★, статус 50/50 и кнопка «Посмотреть» (таблица — этап 5).
-->
<script lang="ts">
  import type { PityCounter, PoolSummary } from '../api/types';
  import { formatNumber } from '../format/format';
  import { i18n } from '../i18n/index.svelte';
  import FeaturedStatusView from './FeaturedStatusView.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Название баннера (уже переведённое). */
    title: string;
    /** Статистика пула из ядра. */
    pool: PoolSummary;
    /** Вызывается по кнопке «Посмотреть». Если не задан — кнопка неактивна. */
    onView?: () => void;
  }

  let { title, pool, onView }: Props = $props();

  /** Подпись «счётчик 34 / 90» под числом «до гаранта». */
  function progress(counter: PityCounter): string {
    return i18n.t('game.pity_progress', { current: counter.current, hard: counter.hard_pity });
  }
</script>

<article class="card">
  <h3>{title}</h3>
  <dl>
    <dt>{i18n.t('game.total_pulls')}</dt>
    <dd>{formatNumber(pool.total_wishes, i18n.language)}</dd>

    <dt>{i18n.t('game.primogems')}</dt>
    <dd class="primogems">{formatNumber(pool.primogems_spent, i18n.language)}</dd>

    <dt>
      {i18n.t('game.pity_5')}
      <span class="hint">{progress(pool.five_star)}</span>
    </dt>
    <dd class="rarity-5">{pool.five_star.remaining}</dd>

    <dt>
      {i18n.t('game.pity_4')}
      <span class="hint">{progress(pool.four_star)}</span>
    </dt>
    <dd class="rarity-4">{pool.four_star.remaining}</dd>
  </dl>

  {#if pool.featured}
    <FeaturedStatusView status={pool.featured} />
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
    display: flex;
    flex-direction: column;
    color: var(--color-text-muted);
  }

  .hint {
    font-size: 11px;
    opacity: 0.8;
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
