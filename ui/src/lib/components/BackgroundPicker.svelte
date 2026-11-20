<!--
  Выбор фона приложения: плитка «Без фона», миниатюры добавленных картинок
  (у каждой — кнопка удаления) и плитка «Добавить».
  Компонент только показывает и сообщает о действиях — данные хранит страница.
-->
<script lang="ts">
  import type { BackgroundImage } from '../api/types';
  import { i18n } from '../i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Добавленные картинки. */
    backgrounds: BackgroundImage[];
    /** Выбранный фон; null — без фона. */
    selected: string | null;
    /** Адрес миниатюры картинки (null — показать заглушку). */
    urlOf: (id: string) => string | null;
    /** Идёт добавление (кнопка «Добавить» неактивна). */
    busy?: boolean;
    /** Выбран фон (null — без фона). */
    onSelect: (id: string | null) => void;
    /** Нажата «Добавить». */
    onAdd: () => void;
    /** Нажато «Удалить» у картинки. */
    onRemove: (id: string) => void;
  }

  let { backgrounds, selected, urlOf, busy = false, onSelect, onAdd, onRemove }: Props = $props();
</script>

<div class="grid">
  <button class="tile none" class:selected={selected === null} aria-pressed={selected === null} onclick={() => onSelect(null)}>
    {i18n.t('background.none')}
  </button>

  {#each backgrounds as background, index (background.id)}
    {@const url = urlOf(background.id)}
    <div class="item">
      <button
        class="tile"
        class:selected={selected === background.id}
        aria-pressed={selected === background.id}
        aria-label={i18n.t('background.select', { number: index + 1 })}
        onclick={() => onSelect(background.id)}
      >
        {#if url}
          <img src={url} alt="" loading="lazy" />
        {:else}
          <span class="placeholder">{index + 1}</span>
        {/if}
      </button>
      <button
        class="remove"
        title={i18n.t('background.remove')}
        aria-label={i18n.t('background.remove')}
        onclick={() => onRemove(background.id)}
      >
        ×
      </button>
    </div>
  {/each}

  <button class="tile add" disabled={busy} onclick={onAdd}>+ {i18n.t('background.add')}</button>
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(144px, 1fr));
    gap: var(--space-3);
  }

  .item {
    position: relative;
  }

  .tile {
    width: 100%;
    aspect-ratio: 16 / 9;
    padding: 0;
    overflow: hidden;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-medium);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .tile:hover {
    background: var(--color-surface-hover);
  }

  .tile.selected {
    border-color: var(--color-accent);
    box-shadow: inset 0 0 0 2px var(--color-accent);
  }

  .tile:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .tile img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .tile.selected img {
    /* Рамка выбора должна быть видна поверх картинки. */
    padding: 2px;
    border-radius: var(--radius-medium);
  }

  .placeholder,
  .none,
  .add {
    color: var(--color-text-muted);
  }

  .remove {
    position: absolute;
    top: var(--space-1);
    right: var(--space-1);
    width: 24px;
    height: 24px;
    padding: 0;
    line-height: 1;
    font-size: 16px;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 50%;
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .item:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }

  .remove:hover {
    color: var(--color-danger);
  }
</style>
