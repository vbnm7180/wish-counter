<!--
  Страница «Импорт и экспорт» игры:
  - профиль игрока: сервер (выпадающий список) и UID;
  - импорт истории из файла Paimon.moe;
  - экспорт истории в файл Paimon.moe.
  Позже здесь появятся импорт из игры и формат UIGF.
-->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import type { ServerRegion } from '../lib/api/types';
  import Card from '../lib/components/Card.svelte';
  import InfoTooltip from '../lib/components/InfoTooltip.svelte';
  import Select from '../lib/components/Select.svelte';
  import { i18n } from '../lib/i18n/index.svelte';
  import { exportErrorKey, gameImportErrorKey, importErrorKey, summaryLines, type MessageLine } from '../lib/import/import-messages';
  import { REGIONS, regionLabelKey } from '../lib/import/regions';

  /** Свойства компонента. */
  interface Props {
    /** Идентификатор игры. */
    gameId: string;
  }

  let { gameId }: Props = $props();

  /** Сообщение под блоком: строки и признак ошибки. */
  type BlockMessage = { lines: MessageLine[]; isError: boolean };

  /** Выбранный сервер; null — не выбран. */
  let region = $state<ServerRegion | null>(null);
  /** UID в поле ввода. */
  let uid = $state('');
  /** Сообщение блока профиля (ошибки сервера/UID, «UID сохранён»). */
  let profileMessage = $state<BlockMessage | null>(null);
  /** Идёт импорт. */
  let importing = $state(false);
  /** Сообщение блока импорта. */
  let importMessage = $state<BlockMessage | null>(null);
  /** Идёт экспорт. */
  let exporting = $state(false);
  /** Сообщение блока экспорта. */
  let exportMessage = $state<BlockMessage | null>(null);
  /** Идёт импорт из игры. */
  let importingFromGame = $state(false);
  /** Сообщение блока импорта из игры. */
  let gameImportMessage = $state<BlockMessage | null>(null);
  /** Идёт очистка данных. */
  let clearing = $state(false);
  /** Сообщение блока очистки. */
  let clearMessage = $state<BlockMessage | null>(null);

  /** Варианты выпадающего списка серверов на текущем языке. */
  let regionOptions = $derived(REGIONS.map((item) => ({ value: item.id, label: i18n.t(item.labelKey) })));

  /** Сообщение из одной строки. */
  function single(key: MessageLine['key'], isError: boolean, params?: MessageLine['params']): BlockMessage {
    return { lines: [{ key, params }], isError };
  }

  /** Загружает профиль игрока (сервер и UID). */
  async function loadProfile(id: string): Promise<void> {
    try {
      const profile = await backend.getGameProfile(id);
      region = profile.server_region;
      uid = profile.uid ?? '';
    } catch (cause) {
      console.error('Не удалось загрузить профиль', cause);
    }
  }

  /** Сохраняет выбранный сервер. */
  async function selectRegion(selected: ServerRegion): Promise<void> {
    profileMessage = null;
    try {
      await backend.setServerRegion(gameId, selected);
      region = selected;
    } catch (cause) {
      console.error('Не удалось сохранить сервер', cause);
      profileMessage = single('region.save_error', true);
    }
  }

  /** Сохраняет введённый UID (проверка — в ядре). */
  async function saveUid(): Promise<void> {
    try {
      await backend.setPlayerUid(gameId, uid);
      profileMessage = single('uid.saved', false);
    } catch (cause) {
      console.error('Не удалось сохранить UID', cause);
      profileMessage = single('uid.invalid', true);
    }
  }

  /** Читает выбранный файл и импортирует его. */
  async function importFile(event: Event & { currentTarget: HTMLInputElement }): Promise<void> {
    const input = event.currentTarget;
    const file = input.files?.[0];
    if (!file) return;
    importing = true;
    importMessage = null;
    try {
      const summary = await backend.importHistory(gameId, 'paimon-moe', await file.text());
      importMessage = { lines: summaryLines(summary, (item) => i18n.t(regionLabelKey(item))), isError: false };
      if (summary.server_region) region = summary.server_region;
      if (summary.uid) uid = summary.uid;
    } catch (cause) {
      console.error('Не удалось импортировать историю', cause);
      importMessage = single(importErrorKey(cause), true);
    } finally {
      importing = false;
      // Сбрасываем выбор, чтобы тот же файл можно было выбрать снова.
      input.value = '';
    }
  }

  /** Экспортирует историю: ядро покажет системный диалог «Сохранить как». */
  async function exportFile(): Promise<void> {
    exporting = true;
    exportMessage = null;
    try {
      const result = await backend.exportHistory(gameId, 'paimon-moe');
      exportMessage = result.path
        ? single('export.saved', false, { count: result.wish_count, path: result.path })
        : single('export.cancelled', false);
    } catch (cause) {
      console.error('Не удалось экспортировать историю', cause);
      exportMessage = single(exportErrorKey(cause), true);
    } finally {
      exporting = false;
    }
  }

  /** Импортирует историю молитв напрямую из API игры. */
  async function importFromGame(): Promise<void> {
    importingFromGame = true;
    gameImportMessage = null;
    try {
      const summary = await backend.importFromGame(gameId);
      const lines: MessageLine[] = [
        { key: 'import.game_result_added', params: { added: summary.added, total: summary.total_in_api } },
      ];
      if (summary.already_present > 0) {
        lines.push({ key: 'import.game_already_present', params: { count: summary.already_present } });
      }
      // Сервер берётся из ссылки игры — без него не определить исход 50/50.
      if (summary.server_region) {
        region = summary.server_region;
        lines.push({ key: 'import.game_result_region', params: { region: i18n.t(regionLabelKey(summary.server_region)) } });
      }
      gameImportMessage = { lines, isError: false };
    } catch (cause) {
      console.error('Не удалось импортировать из игры', cause);
      gameImportMessage = single(gameImportErrorKey(cause), true);
    } finally {
      importingFromGame = false;
    }
  }

  /** Очищает все данные игры после подтверждения. */
  async function clearGameData(): Promise<void> {
    if (!confirm(i18n.t('import.clear_confirm'))) return;
    clearing = true;
    clearMessage = null;
    try {
      await backend.clearGameData(gameId);
      // Профиль удалён вместе с историей — сбросить сервер и UID на странице.
      region = null;
      uid = '';
      clearMessage = single('import.clear_done', false);
    } catch (cause) {
      console.error('Не удалось очистить данные игры', cause);
      clearMessage = single('import.clear_failed', true);
    } finally {
      clearing = false;
    }
  }

  $effect(() => {
    void loadProfile(gameId);
  });
</script>

{#snippet message(block: BlockMessage | null)}
  {#if block}
    <div class="result" class:error={block.isError}>
      {#each block.lines as line (line.key)}
        <p>{i18n.t(line.key, line.params)}</p>
      {/each}
    </div>
  {/if}
{/snippet}

<section class="page">
  <Card isTopLevelCard={true}>
    <h1>{i18n.t('import.heading', { game: i18n.gameName(gameId) })}</h1>

    <Card>
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
      <div class="row">
        <span class="label">{i18n.t('uid.label')}</span>
        <input
          class="uid"
          type="text"
          inputmode="numeric"
          aria-label={i18n.t('uid.label')}
          placeholder={i18n.t('uid.placeholder')}
          bind:value={uid}
        />
        <button class="secondary" onclick={() => void saveUid()}>{i18n.t('uid.save')}</button>
        <InfoTooltip text={i18n.t('uid.hint')} />
      </div>
      {@render message(profileMessage)}
    </Card>

    <Card>
      <h2>{i18n.t('import.paimon_title')}</h2>
      <p class="muted">{i18n.t('import.paimon_steps')}</p>
      <label class="primary" class:disabled={importing}>
        {i18n.t(importing ? 'import.importing' : 'import.choose_file')}
        <input type="file" accept=".json,application/json" disabled={importing} onchange={importFile} />
      </label>
      {@render message(importMessage)}
    </Card>

    <Card>
      <h2>{i18n.t('import.game_title')}</h2>
      <p class="muted">{i18n.t('import.game_description')}</p>
      <button class="primary" class:disabled={importingFromGame} disabled={importingFromGame} onclick={() => void importFromGame()}>
        {i18n.t(importingFromGame ? 'import.game_importing' : 'import.game_button')}
      </button>
      {@render message(gameImportMessage)}
    </Card>

    <Card>
      <h2>{i18n.t('export.title')}</h2>
      <p class="muted">{i18n.t('export.description')}</p>
      <button class="primary" class:disabled={exporting} disabled={exporting} onclick={() => void exportFile()}>
        {i18n.t(exporting ? 'export.saving' : 'export.button')}
      </button>
      {@render message(exportMessage)}
    </Card>

    <Card isDanger={true}>
      <h2>{i18n.t('import.clear_title')}</h2>
      <p class="muted">{i18n.t('import.clear_description')}</p>
      <button class="danger" class:disabled={clearing} disabled={clearing} onclick={() => void clearGameData()}>
        {i18n.t('import.clear_button')}
      </button>
      {@render message(clearMessage)}
    </Card>
  </Card>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    max-width: 720px;
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
    min-width: 64px;
    color: var(--color-text-muted);
  }

  .muted {
    color: var(--color-text-muted);
  }

  .uid {
    width: 180px;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
  }

  .uid:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .primary {
    align-self: flex-start;
    padding: var(--space-2) var(--space-4);
    font: inherit;
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    border: none;
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .secondary {
    padding: var(--space-2) var(--space-3);
    font: inherit;
    background: var(--color-surface-hover);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .primary.disabled {
    opacity: 0.6;
    cursor: progress;
  }

  .danger {
    padding: var(--space-2) var(--space-4);
    background: var(--color-danger);
    color: #fff;
    border: none;
    border-radius: var(--radius-small);
    cursor: pointer;
    font-weight: 600;
    align-self: flex-start;
  }

  .danger.disabled {
    opacity: 0.6;
    cursor: progress;
  }

  /* Стандартное поле выбора файла скрыто: вместо него — кнопка-подпись. */
  .primary input {
    display: none;
  }

  .result {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    overflow-wrap: anywhere;
  }

  .error {
    color: var(--color-danger);
  }
</style>
