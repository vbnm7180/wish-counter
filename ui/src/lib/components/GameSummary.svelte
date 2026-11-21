<!--
  Итоги по игре: всего молитв и потрачено примогемов на всех баннерах.
-->
<script lang="ts">
  import { formatNumber } from '../format/format';
  import { i18n } from '../i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Всего молитв на всех баннерах. */
    totalWishes: number;
    /** Всего потрачено примогемов. */
    primogemsSpent: number;
  }

  let { totalWishes, primogemsSpent }: Props = $props();
</script>

<div class="summary">
  <div class="tile">
    <span class="label">{i18n.t('game.summary_total')}</span>
    <span class="value">{formatNumber(totalWishes, i18n.language)}</span>
  </div>
  <div class="tile">
    <span class="label">{i18n.t('game.summary_primogems')}</span>
    <span class="value primogems">{formatNumber(primogemsSpent, i18n.language)}</span>
  </div>
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .summary {
    display: flex;
    gap: var(--space-4);
    flex-wrap: wrap;
  }

  .tile {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 200px;
    padding: var(--space-3) var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-large);
    box-shadow: var(--shadow-card);
  }

  .label {
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .value {
    font-size: 24px;
    font-weight: 700;
  }

  .primogems {
    color: var(--color-accent);
  }
</style>
