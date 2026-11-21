<!-- Главная страница: приветствие. Позже здесь будет сводка по всем играм. -->
<script lang="ts">
  import { i18n } from '../lib/i18n/index.svelte';
</script>

<section class="page">
  <div class="card">
    <h2 class="page-heading">{i18n.t('home.heading')}</h2>
    <p class="subtitle text-stroke">{i18n.t('home.description')}</p>
  </div>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    box-shadow: var(--shadow-card);
  }

  .page-heading {
    font-size: 26px;
  }

  .subtitle {
    margin: 0;
    font-size: 16px;
    color: var(--color-text-muted);
  }

  .card {
    display: flex;
    flex-direction: column;
    width: fit-content;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-large);
  }

  /* При активном фоне текст подзаголовка берёт основной цвет и получает обводку — иначе не читается. */
  :root[data-background='on'] .subtitle {
    color: var(--color-text);
  }
</style>
