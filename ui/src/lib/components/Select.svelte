<!--
  Выпадающий список. Основан на стандартном <select>: он сам поддерживает
  клавиатуру, экранный диктор и системный вид списка; оформление — через тему.
-->
<script lang="ts" generics="T extends string">
  // `generics="T extends string"` — компонент обобщённый: тип значения задаёт
  // тот, кто его использует (например, ServerRegion), и TypeScript это проверит.

  /** Вариант списка. */
  interface Option {
    /** Значение варианта. */
    value: T;
    /** Подпись (уже переведённая). */
    label: string;
  }

  /** Свойства компонента. */
  interface Props {
    /** Варианты. */
    options: Option[];
    /** Выбранное значение; null — ничего не выбрано (показывается placeholder). */
    value: T | null;
    /** Текст, пока ничего не выбрано. */
    placeholder: string;
    /** Подпись для экранного диктора. */
    label: string;
    /** Вызывается при выборе варианта. */
    onChange: (value: T) => void;
  }

  let { options, value, placeholder, label, onChange }: Props = $props();

  /** Обрабатывает выбор: передаёт значение только если оно из списка вариантов. */
  function handleChange(event: Event & { currentTarget: HTMLSelectElement }): void {
    const selected = options.find((option) => option.value === event.currentTarget.value);
    if (selected) onChange(selected.value);
  }
</script>

<select class="select" aria-label={label} value={value ?? ''} onchange={handleChange}>
  {#if value === null}
    <option value="" disabled>{placeholder}</option>
  {/if}
  {#each options as option (option.value)}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .select {
    min-width: 180px;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .select:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }
</style>
