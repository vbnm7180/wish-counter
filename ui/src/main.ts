// Точка входа интерфейса: подключает стили тем и монтирует корневой компонент.
import { mount } from 'svelte';
import './lib/themes/base.css';
import './lib/themes/dark.css';
import './lib/themes/light.css';
import App from './App.svelte';

const target = document.getElementById('app');
if (!target) {
  throw new Error('Не найден элемент #app в index.html');
}

export default mount(App, { target });
