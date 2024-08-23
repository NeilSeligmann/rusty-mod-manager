import devtools from '@vue/devtools';
import { createApp } from 'vue';
import App from './AppInstaller.vue';
import vuetify from '../plugins/vuetify';
import { createPinia } from 'pinia';
// import router from './plugins/router';
// import { routes } from 'vue-router/auto/routes'

import '../index.css';

if (process.env.NODE_ENV === 'development') {
	devtools.connect('http://localhost', 8098);
}

const vue = createApp(App, { name: 'Installer' });
vue.use(vuetify);

const pinia = createPinia();
// pinia.use(({ store }) => { store.router = markRaw(router) });
vue.use(pinia);

// vue.use(router);

vue.mount('#app');
