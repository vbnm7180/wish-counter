<!--
  Строка «4★ с баннера» в карточке: гарант после проигрыша или шанс (50% / 75%).
  Если статус неизвестен, ничего не показывает: причину уже объясняет блок 5★.
-->
<script lang="ts">
  import type { FeaturedStatus } from '../api/types';
  import { formatChance } from '../format/format';
  import { i18n } from '../i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Статус «4★ с баннера» из ядра. */
    status: FeaturedStatus;
  }

  let { status }: Props = $props();
</script>

{#if status.state === 'known'}
  <div class="four-star">
    {#if status.guaranteed}
      <span class="guaranteed">{i18n.t('featured.four_guaranteed')}</span>
    {:else}
      <span class="label">{i18n.t('featured.four_chance')}</span>
      <span class="chance">{formatChance(status.next_featured_chance, i18n.language)}</span>
    {/if}
  </div>
{/if}

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .four-star {
    display: flex;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .label {
    color: var(--color-text-muted);
  }

  .chance,
  .guaranteed {
    font-weight: 600;
    color: var(--color-rarity-4);
  }
</style>
