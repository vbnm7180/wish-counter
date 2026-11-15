<!--
  Блок статуса 50/50 в карточке баннера персонажа:
  заголовок (50/50, гарант или Capturing Radiance), шанс персонажа с баннера,
  серия проигрышей и правило Capturing Radiance.
-->
<script lang="ts">
  import type { FeaturedStatus } from '../api/types';
  import { formatChance } from '../format/format';
  import { i18n } from '../i18n/index.svelte';
  import { featuredHeadlineKey, radianceThreshold, showLossesUntilRadiance } from '../stats/featured';

  /** Свойства компонента. */
  interface Props {
    /** Статус 50/50 из ядра. */
    status: FeaturedStatus;
    /** Выбран ли сервер игрока (от этого зависит текст при неизвестном статусе). */
    serverChosen: boolean;
  }

  let { status, serverChosen }: Props = $props();
</script>

<section class="featured">
  {#if status.state === 'unknown'}
    <p class="muted">{i18n.t(serverChosen ? 'featured.unknown' : 'featured.unknown_no_region')}</p>
  {:else}
    {@const threshold = radianceThreshold(status)}
    <p class="headline" class:certain={status.chance_source !== 'base'}>
      {i18n.t(featuredHeadlineKey(status))}
    </p>
    <dl>
      <dt>{i18n.t('featured.chance')}</dt>
      <dd class="chance">{formatChance(status.next_featured_chance, i18n.language)}</dd>
      <dt>{i18n.t('featured.losing_streak')}</dt>
      <dd>{status.losing_streak}</dd>
      {#if showLossesUntilRadiance(status)}
        <dt>{i18n.t('featured.until_radiance')}</dt>
        <dd>{status.losses_until_radiance}</dd>
      {/if}
    </dl>
    {#if threshold !== null}
      <p class="muted rule">{i18n.t('featured.radiance_rule', { count: threshold })}</p>
    {/if}
  {/if}
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .featured {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px dashed var(--color-border);
  }

  p {
    margin: 0;
  }

  .headline {
    font-weight: 600;
  }

  .headline.certain {
    color: var(--color-rarity-5);
  }

  dl {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-1) var(--space-4);
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

  .chance {
    color: var(--color-rarity-5);
  }

  .muted {
    color: var(--color-text-muted);
  }

  .rule {
    font-size: 12px;
  }
</style>
