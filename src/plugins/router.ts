import { createRouter, createWebHistory } from 'vue-router/auto';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
import { appWindow } from '@tauri-apps/api/window';
// import Vue from 'vue';

const router = createRouter({
	history: createWebHistory()
	// You don't need to pass the routes anymore,
	// the plugin writes it for you 🤖
	// routes: []
});

router.beforeEach((to, from) => {
	const applicationStateStore = useApplicationStateStore();
	if (applicationStateStore.applicationState.selected_instance) {
		const isEditing = to.path.includes('/instances/current/settings');

		// If we are in settings, mark as editing
		applicationStateStore.applicationConfig.isEditing = isEditing;
		applicationStateStore.instanceConfig.isEditing = isEditing;
	} else {
		// No instance selected!
		// console.log(to.path.split('/')[0]);

		const splitted = to.path.split('/');
		let firstSegment = splitted[0];
		if ((firstSegment === '' || !firstSegment) && splitted.length > 1) {
			firstSegment = splitted[1];
		}

		if (firstSegment !== 'instances') {
			console.log('No instance selected! Redirecting to "/instances"');
			router.push('/instances');
		}
	}
});

router.afterEach((to, from) => {
	const applicationStateStore = useApplicationStateStore();

	// Get instance name, if any
	let selectedInstance = applicationStateStore.applicationState?.selected_instance?.config.name ?? '';
	if (selectedInstance) selectedInstance += ' - ';

	// Get title, if any
	let toRouteTitle = to.meta.title;
	if (toRouteTitle) toRouteTitle += ' - ';
	else toRouteTitle = '';

	appWindow.setTitle(`${toRouteTitle}${selectedInstance}Rusty Mod Manager`);
});

export default router;
