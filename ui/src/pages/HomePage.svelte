<!-- Главная страница: приветствие. Позже здесь будет сводка по всем играм. -->
<script lang="ts">
import Card from '../lib/components/Card.svelte';
import { i18n } from '../lib/i18n/index.svelte';
</script>

<section class="page">
  <Card isTopLevelCard={true}>
    <h2 class="page-heading">{i18n.t('home.heading')}</h2>
    <p class="subtitle text-stroke">{i18n.t('home.description')}</p>
  </Card>
</section>

<!-- Стили компонента: Svelte автоматически ограничивает их этим компонентом. -->
<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    width: fit-content;
  }

  .page-heading {
    font-size: 26px;
  }

  .subtitle {
    margin: 0;
    font-size: 16px;
    color: var(--color-text-muted);
  }

  /* При активном фоне текст подзаголовка берёт основной цвет и получает обводку — иначе не читается. */
  :root[data-background='on'] .subtitle {
    color: var(--color-text);
  }
</style>
