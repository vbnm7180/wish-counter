<!--
  Сводка о справочнике одной игры: количество персонажей, оружия и баннеров,
  последний баннер, версия данных Paimon.moe.
-->
<script lang="ts">
  import { backend } from '../api/backend';
  import type { ReferenceSummary } from '../api/types';
  import { i18n } from '../i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Идентификатор игры. */
    gameId: string;
  }

  let { gameId }: Props = $props();

  /** Длина короткой формы ревизии (как в git: первые 7 символов). */
  const SHORT_REVISION_LENGTH = 7;

  /** Сводка; null — ещё загружается. */
  let summary = $state<ReferenceSummary | null>(null);
  /** Загрузка не удалась. */
  let failed = $state(false);

  /** Загружает сводку о справочнике из ядра. */
  async function load(id: string): Promise<void> {
    try {
      summary = await backend.getReferenceSummary(id);
    } catch (cause) {
      console.error('Не удалось загрузить сводку о справочнике', cause);
      failed = true;
    }
  }

  /** Оставляет от 'ГГГГ-ММ-ДД чч:мм:сс' только дату. */
  function dateOnly(timestamp: string): string {
    return timestamp.split(' ')[0] ?? timestamp;
  }

  $effect(() => {
    void load(gameId);
  });
</script>

<div class="reference">
  <h3>{i18n.gameName(gameId)}</h3>
  {#if failed}
    <p class="error">{i18n.t('error.load_failed')}</p>
  {:else if summary}
    <p>
      {i18n.t('reference.counts', {
        characters: summary.character_count,
        weapons: summary.weapon_count,
        banners: summary.banner_count,
      })}
    </p>
    {#if summary.latest_banner}
      <p>
        {i18n.t('reference.latest_banner', {
          name: summary.latest_banner.short_name,
          version: summary.latest_banner.version ?? '—',
          start: dateOnly(summary.latest_banner.start),
        })}
      </p>
    {/if}
    <p class="muted">
      {i18n.t('reference.source', {
        date: dateOnly(summary.generated_at),
        revision: summary.source_revision.slice(0, SHORT_REVISION_LENGTH),
      })}
    </p>
  {/if}
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .reference {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3) var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-medium);
  }

  h3 {
    font-size: 14px;
    margin-bottom: var(--space-1);
  }

  p {
    margin: 0;
  }

  .muted {
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .error {
    color: var(--color-danger);
  }
</style>
