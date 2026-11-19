<!--
  Страница «Импорт» игры: выбор сервера и импорт истории молитв.
  Сейчас — из файла Paimon.moe; позже здесь появятся UIGF и импорт из игры.
-->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import type { ServerRegion } from '../lib/api/types';
  import InfoTooltip from '../lib/components/InfoTooltip.svelte';
  import Select from '../lib/components/Select.svelte';
  import { i18n } from '../lib/i18n/index.svelte';
  import { importErrorKey, summaryLines, type MessageLine } from '../lib/import/import-messages';
  import { REGIONS, regionLabelKey } from '../lib/import/regions';

  /** Свойства компонента. */
  interface Props {
    /** Идентификатор игры. */
    gameId: string;
  }

  let { gameId }: Props = $props();

  /** Выбранный сервер; null — не выбран. */
  let region = $state<ServerRegion | null>(null);
  /** Идёт импорт. */
  let importing = $state(false);
  /** Строки результата импорта и признак ошибки. */
  let result = $state<{ lines: MessageLine[]; isError: boolean } | null>(null);
  /** Не удалось сохранить сервер. */
  let regionSaveFailed = $state(false);

  /** Варианты выпадающего списка серверов на текущем языке. */
  let regionOptions = $derived(REGIONS.map((item) => ({ value: item.id, label: i18n.t(item.labelKey) })));

  /** Загружает сохранённый сервер. */
  async function loadRegion(id: string): Promise<void> {
    try {
      region = await backend.getServerRegion(id);
    } catch (cause) {
      console.error('Не удалось загрузить сервер', cause);
    }
  }

  /** Сохраняет выбранный сервер. */
  async function selectRegion(selected: ServerRegion): Promise<void> {
    regionSaveFailed = false;
    try {
      await backend.setServerRegion(gameId, selected);
      region = selected;
    } catch (cause) {
      console.error('Не удалось сохранить сервер', cause);
      regionSaveFailed = true;
    }
  }

  /** Читает выбранный файл и импортирует его. */
  async function importFile(event: Event & { currentTarget: HTMLInputElement }): Promise<void> {
    const input = event.currentTarget;
    const file = input.files?.[0];
    if (!file) return;
    importing = true;
    result = null;
    try {
      const summary = await backend.importHistory(gameId, 'paimon-moe', await file.text());
      result = { lines: summaryLines(summary, (item) => i18n.t(regionLabelKey(item))), isError: false };
      if (summary.server_region) region = summary.server_region;
    } catch (cause) {
      console.error('Не удалось импортировать историю', cause);
      result = { lines: [{ key: importErrorKey(cause) }], isError: true };
    } finally {
      importing = false;
      // Сбрасываем выбор, чтобы тот же файл можно было выбрать снова.
      input.value = '';
    }
  }

  $effect(() => {
    void loadRegion(gameId);
  });
</script>

<section class="page">
  <h1>{i18n.t('import.heading', { game: i18n.gameName(gameId) })}</h1>

  <div class="card">
    <div class="row">
      <span class="label">{i18n.t('region.label')}</span>
      <Select
        options={regionOptions}
        value={region}
        placeholder={i18n.t('region.placeholder')}
        label={i18n.t('region.label')}
        onChange={(selected) => void selectRegion(selected)}
      />
      <InfoTooltip text={i18n.t('region.hint')} />
    </div>
    {#if regionSaveFailed}
      <p class="error">{i18n.t('region.save_error')}</p>
    {/if}
  </div>

  <div class="card">
    <h2>{i18n.t('import.paimon_title')}</h2>
    <p class="muted">{i18n.t('import.paimon_steps')}</p>
    <label class="file-button" class:disabled={importing}>
      {i18n.t(importing ? 'import.importing' : 'import.choose_file')}
      <input type="file" accept=".json,application/json" disabled={importing} onchange={importFile} />
    </label>
    {#if result}
      <div class="result" class:error={result.isError}>
        {#each result.lines as line (line.key)}
          <p>{i18n.t(line.key, line.params)}</p>
        {/each}
      </div>
    {/if}
  </div>

  <p class="muted">{i18n.t('import.more_soon')}</p>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    max-width: 720px;
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
    font-size: 16px;
  }

  p {
    margin: 0;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .label {
    color: var(--color-text-muted);
  }

  .muted {
    color: var(--color-text-muted);
  }

  .file-button {
    align-self: flex-start;
    padding: var(--space-2) var(--space-4);
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .file-button.disabled {
    opacity: 0.6;
    cursor: progress;
  }

  /* Стандартное поле выбора файла скрыто: вместо него — кнопка-подпись выше. */
  .file-button input {
    display: none;
  }

  .result {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .error {
    color: var(--color-danger);
  }
</style>
