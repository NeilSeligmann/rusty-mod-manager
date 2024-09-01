import devtools from '@vue/devtools';
import { createApp } from 'vue';
import App from './App.vue';
import vuetify from './plugins/vuetify';
import { createPinia } from 'pinia';
import router from './plugins/router';
import ContextMenu from '@imengyu/vue3-context-menu';
import '@imengyu/vue3-context-menu/lib/vue3-context-menu.css';
import './index.css';

if (process.env.NODE_ENV === 'development') {
	devtools.connect('http://localhost', 8098);
}

const vue = createApp(App);
vue.use(vuetify);

const pinia = createPinia();
vue.use(pinia);

vue.use(router);

vue.use(ContextMenu);

vue.mount('#app');
