<!--
  Картинка предмета (персонажа или оружия) с рамкой по редкости.
  Картинка загружается лениво (только когда видна на экране). Если её нет
  или нет интернета при первом показе — показывается заглушка с первой буквой названия.
-->
<script lang="ts">
  import { itemImageUrl } from '../images/item-image';

  /** Свойства компонента. */
  interface Props {
    /** Игра. */
    gameId: string;
    /** Идентификатор предмета. */
    itemId: string;
    /** Название (подсказка при наведении и буква заглушки). */
    name: string;
    /** Редкость для цвета рамки; null — без цвета. */
    rarity?: 3 | 4 | 5 | null;
    /** Размер в пикселях. */
    size?: number;
  }

  /** Размер иконки по умолчанию. */
  const DEFAULT_SIZE = 48;

  let { gameId, itemId, name, rarity = null, size = DEFAULT_SIZE }: Props = $props();

  /** Картинку загрузить не удалось — показать заглушку. */
  let failed = $state(false);
  /** Адрес картинки; null — протокол недоступен (браузер). */
  let source = $derived(itemImageUrl(gameId, itemId));

  // Новый предмет — новая попытка загрузки.
  $effect(() => {
    void itemId;
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
  {#if source && !failed}
    <img src={source} alt={name} loading="lazy" onerror={() => (failed = true)} />
  {:else}
    <span class="placeholder">{name.charAt(0)}</span>
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
    color: var(--color-text-muted);
  }
</style>
