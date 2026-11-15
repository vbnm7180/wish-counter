<!--
  Страница игры: карточки баннеров со статистикой молитв.
  Сейчас это заготовка: список баннеров задан здесь, значения пустые.
  На этапе 5 баннеры и статистика будут приходить из Rust-ядра.
-->
<script lang="ts">
  import BannerCard from '../lib/components/BannerCard.svelte';
  import { i18n, type TranslationKey } from '../lib/i18n/index.svelte';

  /** Свойства компонента. */
  interface Props {
    /** Идентификатор игры, например 'genshin'. */
    gameId: string;
  }

  let { gameId }: Props = $props();

  /** ВРЕМЕННО: баннеры Genshin Impact до появления данных из ядра. */
  const PLACEHOLDER_BANNERS: TranslationKey[] = [
    'genshin.banner.character',
    'genshin.banner.weapon',
    'genshin.banner.standard',
    'genshin.banner.chronicled',
    'genshin.banner.beginner',
  ];
</script>

<section class="page">
  <h1>{i18n.gameName(gameId)}</h1>
  <p class="hint">{i18n.t('game.placeholder')}</p>

  <h2>{i18n.t('game.banners')}</h2>
  <div class="grid">
    {#each PLACEHOLDER_BANNERS as bannerKey (bannerKey)}
      <BannerCard title={i18n.t(bannerKey)} totalPulls={null} pullsUntilFiveStar={null} pullsUntilFourStar={null} />
    {/each}
  </div>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  h2 {
    font-size: 16px;
    margin-top: var(--space-2);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--space-4);
  }
</style>
