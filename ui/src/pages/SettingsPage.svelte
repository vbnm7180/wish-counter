<!-- Страница настроек: тема оформления, язык интерфейса, фон, сведения о справочных данных. -->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import type { BackgroundImage, GameInfo, LanguageCode } from '../lib/api/types';
  import { backgroundErrorKey } from '../lib/backgrounds/background';
  import BackgroundPicker from '../lib/components/BackgroundPicker.svelte';
  import ReferenceInfo from '../lib/components/ReferenceInfo.svelte';
  import { i18n, LANGUAGES, type TranslationKey } from '../lib/i18n/index.svelte';
  import { backgroundImageUrl } from '../lib/images/item-image';
  import { settingsState } from '../lib/state/settings.svelte';
  import { THEMES } from '../lib/themes/registry';

  /** Свойства компонента. */
  interface Props {
    /** Поддерживаемые игры (для блока справочных данных). */
    games: GameInfo[];
  }

  let { games }: Props = $props();

  /** Выбирает тему оформления. */
  function selectTheme(themeId: string): void {
    void settingsState.update({ theme: themeId });
  }

  /** Выбирает язык интерфейса. */
  function selectLanguage(language: LanguageCode): void {
    void settingsState.update({ language });
  }

  /** Добавленные картинки фона. */
  let backgrounds = $state<BackgroundImage[]>([]);
  /** Идёт добавление картинки (открыт диалог выбора файла). */
  let adding = $state(false);
  /** Ключ текста последней ошибки работы с фонами или null. */
  let backgroundError = $state<TranslationKey | null>(null);

  /** Загружает список фонов. */
  async function loadBackgrounds(): Promise<void> {
    try {
      backgrounds = await backend.listBackgrounds();
    } catch (cause) {
      console.error('Не удалось загрузить фоны', cause);
      backgroundError = 'background.load_failed';
    }
  }

  /** Выбирает фон (null — без фона). */
  function selectBackground(id: string | null): void {
    void settingsState.update({ background: id });
  }

  /** Добавляет картинку через диалог выбора файла и сразу делает её фоном. */
  async function addBackground(): Promise<void> {
    adding = true;
    backgroundError = null;
    try {
      const added = await backend.addBackground();
      if (added) {
        await loadBackgrounds();
        selectBackground(added.id);
      }
    } catch (cause) {
      console.error('Фон не добавлен', cause);
      backgroundError = backgroundErrorKey(cause);
    } finally {
      adding = false;
    }
  }

  /** Удаляет картинку фона (если она выбрана — фон сбрасывается в ядре). */
  async function removeBackground(id: string): Promise<void> {
    backgroundError = null;
    try {
      settingsState.replace(await backend.removeBackground(id));
      await loadBackgrounds();
    } catch (cause) {
      console.error('Фон не удалён', cause);
      backgroundError = backgroundErrorKey(cause);
    }
  }

  $effect(() => {
    void loadBackgrounds();
  });
</script>

<section class="page">
  <h1 class="page-heading">{i18n.t('settings.heading')}</h1>

  <div class="card">
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

  <div class="card">
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

  <div class="card">
    <h2>{i18n.t('settings.background')}</h2>
    <BackgroundPicker
      {backgrounds}
      selected={settingsState.current.background}
      urlOf={backgroundImageUrl}
      busy={adding}
      onSelect={selectBackground}
      onAdd={() => void addBackground()}
      onRemove={(id) => void removeBackground(id)}
    />
    <p class="hint">{i18n.t('background.hint')}</p>
    {#if backgroundError}
      <p class="error">{i18n.t(backgroundError)}</p>
    {/if}
  </div>

  <div class="card">
    <h2>{i18n.t('settings.reference')}</h2>
    {#each games as game (game.id)}
      <ReferenceInfo gameId={game.id} />
    {/each}
    <p class="license">{i18n.t('reference.license')}</p>
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

  .card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-large);
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

  .hint,
  .license {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
  }
</style>
