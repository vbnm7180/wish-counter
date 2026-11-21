<!--
  Сводка о справочнике одной игры: количество персонажей, оружия и баннеров,
  последний баннер с картинками персонажей, версия данных Paimon.moe, время последней проверки
  и кнопка «Обновить данные».
-->
<script lang="ts">
  import { backend } from '../api/backend';
  import { onReferenceUpdated } from '../api/events';
  import type { ReferenceSummary } from '../api/types';
  import { i18n, type TranslationKey } from '../i18n/index.svelte';
  import { errorMessageKey, outcomeMessageKey } from '../reference/update-messages';
  import ItemIcon from './ItemIcon.svelte';

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
  /** Идёт проверка обновления. */
  let updating = $state(false);
  /** Результат последней проверки по кнопке: текст и признак ошибки. */
  let result = $state<{ key: TranslationKey; isError: boolean } | null>(null);

  /** Загружает сводку о справочнике из ядра. */
  async function load(id: string): Promise<void> {
    try {
      summary = await backend.getReferenceSummary(id);
      failed = false;
    } catch (cause) {
      console.error('Не удалось загрузить сводку о справочнике', cause);
      failed = true;
    }
  }

  /** Проверяет обновление по кнопке и показывает результат. */
  async function update(): Promise<void> {
    updating = true;
    result = null;
    try {
      const outcome = await backend.updateReference(gameId);
      result = { key: outcomeMessageKey(outcome), isError: false };
      await load(gameId);
    } catch (cause) {
      console.error('Не удалось обновить справочник', cause);
      result = { key: errorMessageKey(cause), isError: true };
    } finally {
      updating = false;
    }
  }

  /** Оставляет от 'ГГГГ-ММ-ДД чч:мм:сс' только дату. */
  function dateOnly(timestamp: string): string {
    return timestamp.split(' ')[0] ?? timestamp;
  }

  $effect(() => {
    void load(gameId);
  });

  // Справочник обновился в фоне (при запуске) — перечитать сводку.
  // Функция, которую возвращает $effect, вызывается при уходе со страницы: отписка.
  $effect(() => {
    const subscription = onReferenceUpdated((updatedGameId) => {
      if (updatedGameId === gameId) void load(gameId);
    });
    return () => void subscription.then((unsubscribe) => unsubscribe());
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
      <div class="featured">
        {#each summary.latest_banner.featured as item (item.id)}
          <ItemIcon
            {gameId}
            itemId={item.id}
            name={item.name ? i18n.localized(item.name) : item.id}
            rarity={5}
          />
        {/each}
      </div>
    {/if}
    <p class="muted">
      {i18n.t('reference.source', {
        date: dateOnly(summary.generated_at),
        revision: summary.source_revision.slice(0, SHORT_REVISION_LENGTH),
      })}
    </p>
    <p class="muted">
      {summary.last_checked_at
        ? i18n.t('reference.last_checked', { date: summary.last_checked_at })
        : i18n.t('reference.never_checked')}
    </p>

    <div class="actions">
      <button class="update" disabled={updating} onclick={() => void update()}>
        {i18n.t(updating ? 'reference.updating' : 'reference.update_button')}
      </button>
      {#if result}
        <span class:error={result.isError}>{i18n.t(result.key)}</span>
      {/if}
    </div>
  {/if}
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .reference {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
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

  .featured {
    display: flex;
    gap: var(--space-2);
    margin: var(--space-1) 0;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }

  .update {
    padding: var(--space-1) var(--space-3);
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    border: none;
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .update:disabled {
    opacity: 0.6;
    cursor: progress;
  }

  .error {
    color: var(--color-danger);
  }
</style>
