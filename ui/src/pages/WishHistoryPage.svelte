<!--
  Таблица молитв одного баннера (кнопка «Посмотреть» на карточке):
  дата, предмет с картинкой, тип, номер по гаранту, исход 50/50, баннер.
  Новые молитвы сверху; фильтры по редкости, типу и названию; по 50 строк на странице.
-->
<script lang="ts">
  import { backend } from '../lib/api/backend';
  import { onHistoryUpdated } from '../lib/api/events';
  import type { FiftyFiftyOutcome, PoolHistory, WishRow } from '../lib/api/types';
  import InfoTooltip from '../lib/components/InfoTooltip.svelte';
  import ItemIcon from '../lib/components/ItemIcon.svelte';
  import Select from '../lib/components/Select.svelte';
  import { defaultFilter, filterRows, HARD_PITY_5, pageCount, pageRows, pityHue, formatWishTime, type KindFilter } from '../lib/history/table';
  import { i18n, type TranslationKey } from '../lib/i18n/index.svelte';
  import { navigation } from '../lib/state/navigation.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Игра. */
    gameId: string;
    /** Пул баннеров. */
    poolId: string;
  }

  let { gameId, poolId }: Props = $props();

  /** Размер картинки предмета в строке таблицы (пиксели). */
  const ICON_SIZE = 36;
  /** Редкости для кнопок-фильтров (сверху вниз по ценности). */
  const RARITIES = [5, 4, 3] as const;
  /** Подписи исходов 50/50. */
  const OUTCOME_KEYS: Record<FiftyFiftyOutcome, TranslationKey> = {
    won: 'history.outcome_won',
    lost: 'history.outcome_lost',
    guaranteed: 'history.outcome_guaranteed',
    unknown: 'history.outcome_unknown',
  };

  /** История; null — загружается. */
  let history = $state<PoolHistory | null>(null);
  /** Загрузка не удалась. */
  let failed = $state(false);
  /** Условия фильтра. */
  let filter = $state(defaultFilter());
  /** Текущая страница (с 1). */
  let page = $state(1);

  /** Название строки на текущем языке (или идентификатор, если предмета нет в справочнике). */
  function nameOf(row: WishRow): string {
    return row.name ? i18n.localized(row.name) : row.item_id;
  }

  /** Строки после фильтра. */
  let filtered = $derived(history ? filterRows(history.rows, filter, nameOf) : []);
  /** Число страниц. */
  let pages = $derived(pageCount(filtered.length));
  /** Строки текущей страницы. */
  let visible = $derived(pageRows(filtered, page));

  /** Варианты фильтра по типу. */
  let kindOptions = $derived<{ value: KindFilter; label: string }[]>([
    { value: 'all', label: i18n.t('history.kind_all') },
    { value: 'character', label: i18n.t('history.kind_character') },
    { value: 'weapon', label: i18n.t('history.kind_weapon') },
  ]);

  /** Загружает историю пула. */
  async function load(): Promise<void> {
    try {
      history = await backend.getPoolHistory(gameId, poolId);
      failed = false;
    } catch (cause) {
      console.error('Не удалось загрузить историю', cause);
      failed = true;
    }
  }

  /** Включает/выключает редкость в фильтре и возвращает на первую страницу. */
  function toggleRarity(rarity: 3 | 4 | 5): void {
    const rarities = new Set(filter.rarities);
    if (rarities.has(rarity)) rarities.delete(rarity);
    else rarities.add(rarity);
    filter = { ...filter, rarities };
    page = 1;
  }

  /** Меняет фильтр по типу. */
  function selectKind(kind: KindFilter): void {
    filter = { ...filter, kind };
    page = 1;
  }

  /** Меняет строку поиска. */
  function search(event: Event & { currentTarget: HTMLInputElement }): void {
    filter = { ...filter, search: event.currentTarget.value };
    page = 1;
  }

  $effect(() => {
    void poolId;
    void load();
  });

  // После импорта история могла измениться — перечитать.
  $effect(() => {
    const subscription = onHistoryUpdated((updatedGameId) => {
      if (updatedGameId === gameId) void load();
    });
    return () => void subscription.then((unsubscribe) => unsubscribe());
  });
</script>

<section class="page">
  <div class="card">
    <div class="header">
      <button class="back" onclick={() => navigation.goTo({ kind: 'game', gameId })}>{i18n.t('history.back')}</button>
      <h1 class="page-heading">{i18n.poolName(gameId, poolId)}</h1>
    </div>

    {#if failed}
      <p class="error">{i18n.t('error.load_failed')}</p>
    {:else if history}
      <div class="table-card">
      <div class="filters">
        <div class="rarities">
          {#each RARITIES as rarity (rarity)}
            <button
              class="rarity rarity-{rarity}"
              class:on={filter.rarities.has(rarity)}
              aria-pressed={filter.rarities.has(rarity)}
              onclick={() => toggleRarity(rarity)}
            >
              {rarity}★
            </button>
          {/each}
        </div>
        <Select
          options={kindOptions}
          value={filter.kind}
          placeholder={i18n.t('history.kind')}
          label={i18n.t('history.kind')}
          onChange={selectKind}
        />
        <input
          class="search"
          type="search"
          placeholder={i18n.t('history.search')}
          aria-label={i18n.t('history.search')}
          value={filter.search}
          oninput={search}
        />
      </div>

      <p class="muted">{i18n.t('history.count', { shown: filtered.length, total: history.rows.length })}</p>

      {#if filtered.length === 0}
        <p class="muted">{i18n.t('history.empty')}</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>
                <span class="th-with-hint">
                  {i18n.t('history.col_time')}
                  <InfoTooltip text={i18n.t('history.time_hint')} />
                </span>
              </th>
              <th>{i18n.t('history.col_item')}</th>
              <th>{i18n.t('history.col_kind')}</th>
              <th class="number">{i18n.t('history.col_pity')}</th>
              <th>{i18n.t('history.col_outcome')}</th>
              <th>{i18n.t('history.col_banner')}</th>
            </tr>
          </thead>
          <tbody>
            {#each visible as row (row.id)}
              {@const hue = pityHue(row.pity, row.rarity, row.rarity === 5 ? (HARD_PITY_5[poolId] ?? 90) : row.rarity === 4 ? 10 : undefined)}
              <tr class="row-{row.rarity}">
                <td class="time">{formatWishTime(row.time)}</td>
                <td>
                  <div class="item">
                    <ItemIcon {gameId} itemId={row.item_id} name={nameOf(row)} rarity={row.rarity} size={ICON_SIZE} />
                    <span class="name">{nameOf(row)}</span>
                  </div>
                </td>
                <td class="muted">{i18n.t(row.item_kind === 'character' ? 'history.character' : 'history.weapon')}</td>
                <!-- style:--pity-hue — CSS-переменная только для этой ячейки; цвет собирается в стилях ниже. -->
                <td class="number pity" class:colored={hue !== null} style:--pity-hue={hue}>{row.pity ?? ''}</td>
                <td>
                  {#if row.outcome}
                    <span class="outcome outcome-{row.outcome}">{i18n.t(OUTCOME_KEYS[row.outcome])}</span>
                  {/if}
                </td>
                <td class="muted">{row.banner_name ?? ''}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        {#if pages > 1}
          <div class="pagination">
            <button disabled={page <= 1} onclick={() => (page -= 1)}>{i18n.t('history.prev')}</button>
            <span>{i18n.t('history.page', { page, pages })}</span>
            <button disabled={page >= pages} onclick={() => (page += 1)}>{i18n.t('history.next')}</button>
          </div>
        {/if}
      {/if}
      </div><!-- /.table-card -->
    {/if}
  </div>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

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

  .header {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .back,
  .pagination button {
    padding: var(--space-1) var(--space-3);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
    cursor: pointer;
  }

  .back {
    width: 43px;
    font-size: 20px;
  }

  .pagination button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
  }

  .rarities {
    display: flex;
    gap: var(--space-1);
  }

  .rarity {
    padding: var(--space-1) var(--space-3);
    font-weight: 600;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
    opacity: 0.45;
    cursor: pointer;
  }

  .rarity.on {
    opacity: 1;
  }

  .rarity-5 {
    color: var(--color-rarity-5);
  }

  .rarity-4 {
    color: var(--color-rarity-4);
  }

  .rarity-3 {
    color: var(--color-rarity-3);
  }

  .search {
    min-width: 220px;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-small);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  .table-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-large);
    box-shadow: var(--shadow-card);
  }

  th {
    padding: var(--space-2) var(--space-3);
    font-size: 12px;
    font-weight: 600;
    text-align: left;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
  }

  td {
    padding: var(--space-1) var(--space-3);
    border-bottom: 1px solid var(--color-border);
  }

  tbody tr:last-child td {
    border-bottom: none;
  }

  /* Строки 4★ и 5★ выделены цветом редкости слева. */
  .row-5 td:first-child {
    box-shadow: inset 3px 0 0 var(--color-rarity-5);
  }

  .row-4 td:first-child {
    box-shadow: inset 3px 0 0 var(--color-rarity-4);
  }

  .row-5 .name {
    font-weight: 600;
    color: var(--color-rarity-5);
  }

  .row-4 .name {
    font-weight: 600;
    color: var(--color-rarity-4);
  }

  .th-with-hint {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .time {
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .number {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .pity {
    font-weight: 700;
  }

  /* Оттенок — из pityHue (0 — красный, 60 — жёлтый, 120 — зелёный), насыщенность и яркость — из темы. */
  .pity.colored {
    color: hsl(var(--pity-hue) var(--pity-saturation) var(--pity-lightness));
  }

  .outcome {
    padding: 2px var(--space-2);
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-small);
    background: var(--color-surface-hover);
  }

  .outcome-won {
    color: var(--color-rarity-5);
  }

  .outcome-lost {
    color: var(--color-danger);
  }

  .outcome-guaranteed {
    color: var(--color-rarity-4);
  }

  .muted {
    margin: 0;
    color: var(--color-text-muted);
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
  }

  .error {
    color: var(--color-danger);
  }
</style>
