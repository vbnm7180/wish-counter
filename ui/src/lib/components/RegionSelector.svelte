<!--
  Выбор сервера игрока. Пока сервер не выбран, показывает подсказку,
  зачем он нужен.
-->
<script lang="ts">
  import type { ServerRegion } from '../api/types';
  import { i18n, type TranslationKey } from '../i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Выбранный сервер; null — не выбран. */
    selected: ServerRegion | null;
    /** Вызывается при выборе сервера. */
    onSelect: (region: ServerRegion) => void;
  }

  let { selected, onSelect }: Props = $props();

  /** Серверы в порядке отображения и ключи их подписей. */
  const REGIONS: { id: ServerRegion; labelKey: TranslationKey }[] = [
    { id: 'america', labelKey: 'region.america' },
    { id: 'europe', labelKey: 'region.europe' },
    { id: 'asia', labelKey: 'region.asia' },
    { id: 'china', labelKey: 'region.china' },
  ];
</script>

<div class="region">
  <span class="label">{i18n.t('region.label')}</span>
  <div class="options">
    {#each REGIONS as region (region.id)}
      <button class="option" class:selected={selected === region.id} onclick={() => onSelect(region.id)}>
        {i18n.t(region.labelKey)}
      </button>
    {/each}
  </div>
  {#if selected === null}
    <p class="hint">{i18n.t('region.hint')}</p>
  {/if}
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .region {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
  }

  .label {
    color: var(--color-text-muted);
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .option {
    padding: var(--space-1) var(--space-3);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .option:hover {
    background: var(--color-surface-hover);
  }

  .option.selected {
    border-color: var(--color-accent);
    box-shadow: inset 0 0 0 1px var(--color-accent);
  }

  .hint {
    flex-basis: 100%;
    margin: 0;
    color: var(--color-accent);
  }
</style>
