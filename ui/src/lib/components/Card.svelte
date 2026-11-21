<!--
  Визуальный компонент - карточка, в которую можно прокинуть содержимое.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    //Является ли карточка верхнеуровневой
    isTopLevelCard?: boolean;
    //Передаваемый в карточку контент
    children: Snippet;
    //Красная граница карточки
    isDanger?: boolean
  }

  let { isTopLevelCard = false, children, isDanger = false }: Props = $props();
</script>

<!-- Если карточка верхнеуровневая - скрываем ее стили без примененного фона -->
<div class="card" class:top-level={isTopLevelCard} class:danger-zone={isDanger}>
  {@render children()}
</div>

<style>
    .card {
      display: flex;
      flex-direction: column;
      gap: var(--space-3);
      padding: var(--space-4);
      background: var(--color-surface);
      border: 1px solid var(--color-border);
      border-radius: var(--radius-large);
      box-shadow: var(--shadow-card);
    }

    .card.danger-zone {
      border-color: var(--color-danger);
    }

    /*
      * Когда выбран фон (data-background="on" у <html>), появляется карточка верхнего уровня
      * Для предотвращения слияния заголовков с фоном.
      * Селектор с [data-theme] сильнее селекторов тем, поэтому переопределяет их цвета.
    */
    :root:not([data-background='on']) .card.top-level {
      border: none;
      box-shadow: none;
      background: transparent;
    }
</style>
