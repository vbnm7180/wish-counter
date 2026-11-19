<!--
  Картинка предмета (персонажа или оружия) с рамкой по редкости.
  Адрес строится по игре и предмету, отображение — в IconImage.
-->
<script lang="ts">
  import { itemImageUrl } from '../images/item-image';
  import IconImage from './IconImage.svelte';

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

  let { gameId, itemId, name, rarity = null, size }: Props = $props();
</script>

<IconImage src={itemImageUrl(gameId, itemId)} {name} {rarity} {size} />
