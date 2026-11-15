<!-- Страница настроек: выбор темы оформления и языка интерфейса. -->
<script lang="ts">
  import type { LanguageCode } from '../lib/api/types';
  import { i18n, LANGUAGES } from '../lib/i18n/index.svelte';
  import { settingsState } from '../lib/state/settings.svelte';
  import { THEMES } from '../lib/themes/registry';

  /** Выбирает тему оформления. */
  function selectTheme(themeId: string): void {
    void settingsState.update({ theme: themeId });
  }

  /** Выбирает язык интерфейса. */
  function selectLanguage(language: LanguageCode): void {
    void settingsState.update({ language });
  }
</script>

<section class="page">
  <h1>{i18n.t('settings.heading')}</h1>

  <div class="group">
    <h2>{i18n.t('settings.theme')}</h2>
    <div class="options">
      {#each THEMES as theme (theme.id)}
        <button
          class="option"
          class:selected={settingsState.current.theme === theme.id}
          onclick={() => selectTheme(theme.id)}
        >
          {i18n.t(theme.labelKey)}
        </button>
      {/each}
    </div>
  </div>

  <div class="group">
    <h2>{i18n.t('settings.language')}</h2>
    <div class="options">
      {#each LANGUAGES as language (language.code)}
        <button
          class="option"
          class:selected={settingsState.current.language === language.code}
          onclick={() => selectLanguage(language.code)}
        >
          {i18n.t(language.labelKey)}
        </button>
      {/each}
    </div>
  </div>

  {#if settingsState.error}
    <p class="error">{i18n.t(settingsState.error)}</p>
  {/if}
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    max-width: 560px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  h2 {
    font-size: 15px;
  }

  .options {
    display: flex;
    gap: var(--space-2);
  }

  .option {
    padding: var(--space-2) var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-medium);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .option:hover {
    background: var(--color-surface-hover);
  }

  .option.selected {
    border-color: var(--color-accent);
    box-shadow: inset 0 0 0 1px var(--color-accent);
  }

  .error {
    margin: 0;
    color: var(--color-danger);
  }
</style>
