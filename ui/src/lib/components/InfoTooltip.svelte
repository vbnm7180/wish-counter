<!--
  Иконка «i» с подсказкой. Подсказка видна при наведении мыши и при фокусе
  с клавиатуры (Tab), а экранный диктор зачитывает её через aria-describedby.
-->
<script lang="ts">
  /** Свойства компонента. */
  interface Props {
    /** Текст подсказки (уже переведённый). */
    text: string;
  }

  let { text }: Props = $props();

  /** Уникальный id подсказки для связи с кнопкой (aria-describedby). */
  const tooltipId = `tooltip-${Math.random().toString(36).slice(2)}`;
</script>

<span class="info">
  <button type="button" class="icon" aria-describedby={tooltipId} aria-label={text}>i</button>
  <span id={tooltipId} role="tooltip" class="tooltip">{text}</span>
</span>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .info {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .icon {
    width: 18px;
    height: 18px;
    padding: 0;
    font-size: 12px;
    font-weight: 700;
    font-style: italic;
    line-height: 1;
    color: var(--color-text-muted);
    background: transparent;
    border: 1.5px solid var(--color-text-muted);
    border-radius: 50%;
    cursor: help;
  }

  .tooltip {
    position: absolute;
    bottom: calc(100% + var(--space-2));
    left: 50%;
    z-index: 10;
    width: max-content;
    max-width: 280px;
    padding: var(--space-2) var(--space-3);
    font-size: 12px;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
    box-shadow: var(--shadow-card);
    transform: translateX(-50%);
    visibility: hidden;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .info:hover .tooltip,
  .icon:focus-visible + .tooltip {
    visibility: visible;
    opacity: 1;
  }
</style>
