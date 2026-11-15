<!--
  Карточка баннера: название, всего молитв, сколько до гарантов,
  кнопка «Посмотреть». Пока данные — заглушки (null → «—»).
-->
<script lang="ts">
  import { i18n } from '../i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Название баннера (уже переведённое). */
    title: string;
    /** Всего молитв на баннере или null, если данных нет. */
    totalPulls: number | null;
    /** Молитв до гаранта 5★ или null. */
    pullsUntilFiveStar: number | null;
    /** Молитв до гаранта 4★ или null. */
    pullsUntilFourStar: number | null;
    /** Вызывается по кнопке «Посмотреть». Если не задан — кнопка неактивна. */
    onView?: () => void;
  }

  let { title, totalPulls, pullsUntilFiveStar, pullsUntilFourStar, onView }: Props = $props();

  /** Форматирует число или показывает прочерк, если значения нет. */
  function display(value: number | null): string {
    return value === null ? '—' : String(value);
  }
</script>

<article class="card">
  <h3>{title}</h3>
  <dl>
    <dt>{i18n.t('game.total_pulls')}</dt>
    <dd>{display(totalPulls)}</dd>
    <dt>{i18n.t('game.pity_5')}</dt>
    <dd class="rarity-5">{display(pullsUntilFiveStar)}</dd>
    <dt>{i18n.t('game.pity_4')}</dt>
    <dd class="rarity-4">{display(pullsUntilFourStar)}</dd>
  </dl>
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

  .rarity-5 {
    color: var(--color-rarity-5);
  }

  .rarity-4 {
    color: var(--color-rarity-4);
  }

  .view {
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
