<!--
  Квадратная картинка (предмета, игры) с рамкой по редкости.
  Загружается лениво (только когда видна на экране). Если адреса нет или
  картинку загрузить не удалось — показывается заглушка с первой буквой названия.
-->
<script lang="ts">
  /** Свойства компонента. */
  interface Props {
    /** Адрес картинки; null — сразу заглушка. */
    src: string | null;
    /** Название (подсказка при наведении и буква заглушки). */
    name: string;
    /** Редкость для цвета рамки; null — обычная рамка. */
    rarity?: 3 | 4 | 5 | null;
    /** Размер в пикселях. */
    size?: number;
  }

  /** Размер по умолчанию. */
  const DEFAULT_SIZE = 48;

  let { src, name, rarity = null, size = DEFAULT_SIZE }: Props = $props();

  /** Картинку загрузить не удалось — показать заглушку. */
  let failed = $state(false);

  // Новый адрес — новая попытка загрузки.
  $effect(() => {
    void src;
    failed = false;
  });
</script>

<div
  class="icon"
  class:rarity-5={rarity === 5}
  class:rarity-4={rarity === 4}
  class:rarity-3={rarity === 3}
  style:width="{size}px"
  style:height="{size}px"
  title={name}
>
  {#if src && !failed}
    <img {src} alt={name} loading="lazy" onerror={() => (failed = true)} />
  {:else}
    <span class="placeholder" style:font-size="{Math.round(size / 2)}px">{name.charAt(0)}</span>
  {/if}
</div>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    overflow: hidden;
    background: var(--color-surface-hover);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-medium);
  }

  .rarity-5 {
    border-color: var(--color-rarity-5);
  }

  .rarity-4 {
    border-color: var(--color-rarity-4);
  }

  .rarity-3 {
    border-color: var(--color-rarity-3);
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .placeholder {
    font-weight: 700;
    line-height: 1;
    color: var(--color-text-muted);
  }
</style>
