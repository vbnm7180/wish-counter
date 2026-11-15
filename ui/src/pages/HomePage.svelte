<!-- Главная страница: приветствие. Позже здесь будет сводка по всем играм. -->
<script lang="ts">
  import { i18n } from '../lib/i18n/index.svelte';
</script>

<section class="page">
  <h1>{i18n.t('home.heading')}</h1>
  <p>{i18n.t('home.description')}</p>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  p {
    margin: 0;
    color: var(--color-text-muted);
  }
</style>
