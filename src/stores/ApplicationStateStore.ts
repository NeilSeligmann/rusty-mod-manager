import { defineStore } from 'pinia';
import { type Ref, ref, reactive } from 'vue';
import { taurpc } from '@/lib/taurpc';
import { emit, listen } from '@tauri-apps/api/event';
import type {
	FrontendConfig,
	ApplicationConfig,
	GameInstanceConfig,
	InstanceExecutable,
	IPCPayload
} from '@/lib/bindings';
import type { ApplicationState } from '@/lib/bindingsFront';
// import { appWindow } from "@tauri-apps/api/window";
// import router from '@/plugins/router';

export const useApplicationStateStore = defineStore('applicationState', () => {
	// @ts-expect-error This object will be populated by the backend
	const applicationState: Ref<Omit<ApplicationState, 'application_config'>> = ref({});
	// @ts-expect-error This object will be populated by the backend
	const applicationConfig: Ref<{ isDirty: boolean; isEditing: boolean; config: ApplicationConfig }> = ref({
		isDirty: false,
		isEditing: false,
		config: {}
	});

	const instanceConfig: Ref<{ isDirty: boolean; isEditing: boolean; config: GameInstanceConfig | null }> = ref({
		isDirty: false,
		isEditing: false,
		config: null
	});
	const isInitialLoad = ref(true);
	const ignoreStateUpdate = ref(false);

	async function updateState(state: ApplicationState) {
		if (applicationConfig.value.isEditing || instanceConfig.value.isEditing) return;

		// If the ignoreStateUpdate flag is set, we dont want to update the state
		if (ignoreStateUpdate.value) {
			return;
		}

		// If we are editing the config, we dont want to overwrite it and lose the changes
		if (!applicationConfig.value.isEditing) {
			applicationConfig.value.config = JSON.parse(JSON.stringify(state.application_config));
			applicationConfig.value.isDirty = false;
		}

		// If we are editing the config, we dont want to overwrite it and lose the changes
		if (!instanceConfig.value.isEditing) {
			instanceConfig.value.config = JSON.parse(JSON.stringify(state.selected_instance?.config || null));
			instanceConfig.value.isDirty = false;
		}

		// Set the application state
		applicationState.value = state;

		isInitialLoad.value = false;
	}

	async function fetchBackendState(withDownloads = false) {
		console.log('Fetching backend state...');
		const state = await taurpc.get_state(withDownloads);
		updateState(state);

		console.log('Fetched backend state', state);

		return state;
	}

	// Fetch initial state
	void fetchBackendState(true);

	// Listen for the on_state_change event
	taurpc.on_state_changed.on(new_state => {
		if (!new_state.application_config) {
			console.error('Received invalid state from backend! Missing "application_config"');
			return;
		}

		if (!new_state.frontend_config) {
			console.error('Received invalid state from backend! Missing "frontend_config"');
			return;
		}

		console.log('Received state from backend', new_state);

		// @ts-expect-error We already checked for the existence of the optional fields
		updateState(new_state);
	});

	taurpc.downloads.on_downloads_update.on(downloads => {
		if (!applicationState.value.selected_instance?.downloads) return;

		for (const download of downloads) {
			const existingDownload = applicationState.value.selected_instance.downloads.find(
				d => d.file_name === download.file_name
			);

			if (existingDownload) {
				Object.assign(existingDownload, download);
			} else {
				applicationState.value.selected_instance.downloads.push(download);
			}
		}
	});

	function getFrontendConfig<K extends keyof FrontendConfig>(key: K) {
		if (!applicationState.value.frontend_config) {
			throw new Error('Frontend config has not been initialized yet!');
		}

		return applicationState.value.frontend_config[key];
	}

	async function updateFrontendConfig<K extends keyof FrontendConfig>(key: K, value: FrontendConfig[K]) {
		if (!applicationState.value.frontend_config) {
			throw new Error('Frontend config has not been initialized yet!');
		}

		const newFrontendConfig = { ...applicationState.value.frontend_config };
		newFrontendConfig[key] = value;

		taurpc.update_frontend_config(newFrontendConfig);
	}

	// Application config
	// We keep a differential copy of the application config in the store
	// so that we can pile up changes and send them all at once

	// const pendingApplicationConfig: Ref<Partial<ApplicationConfig>> = ref({});
	// const applicationConfig = new Proxy({} as ApplicationConfig, {
	// 	get(target, prop: keyof ApplicationConfig, receiver) {
	// 		// If the property is in the diff, return that value
	// 		// if (typeof pendingApplicationConfigDiff.value[prop] !== 'undefined') {
	// 		// 	return pendingApplicationConfigDiff.value[prop];
	// 		// }

	// 		if (typeof applicationState.value.application_config === 'undefined') {
	// 			return undefined;
	// 		}
	// 		return Reflect.get(applicationState.value.application_config, prop, receiver);
	// 		// return applicationState.value.application_config[prop];
	// 	},
	// 	set(target, prop: keyof ApplicationConfig, value) {
	// 		// Set the value in the diff
	// 		// pendingApplicationConfigDiff.value[prop] = value;
	// 		return Reflect.set(pendingApplicationConfigDiff, prop, value);

	// 		// Indicate success
	// 		// return true;
	// 	}
	// });

	// -------------------
	// Application config
	// -------------------

	watch(
		applicationConfig,
		() => {
			const A = JSON.stringify(applicationConfig.value.config);
			// @ts-expect-error
			const B = JSON.stringify(applicationState.value.application_config);

			applicationConfig.value.isDirty = A !== B;
		},
		{ deep: true }
	);

	async function saveApplicationConfig() {
		// Send the diff to the backend
		await taurpc.update_application_config(applicationConfig.value.config);

		applicationConfig.value.isDirty = false;

		// Clear the diff
		// pendingApplicationConfig.value = {};
	}

	// -------------------
	// Instance config
	// -------------------
	watch(
		instanceConfig,
		() => {
			const A = JSON.stringify(instanceConfig.value.config);
			const B = JSON.stringify(applicationState.value.selected_instance?.config || null);

			instanceConfig.value.isDirty = A !== B;
		},
		{ deep: true }
	);

	async function saveInstanceConfig() {
		if (!instanceConfig.value.config) {
			throw new Error('Instance config is not initialized yet!');
		}

		// Send the diff to the backend
		await taurpc.instances.update_config(instanceConfig.value.config);
	}

	async function saveAllConfigs() {
		ignoreStateUpdate.value = true;
		if (applicationConfig.value.isDirty) await saveApplicationConfig();
		if (instanceConfig.value.isDirty) await saveInstanceConfig();
		ignoreStateUpdate.value = false;

		// Fetch the backend state to update the store
		await fetchBackendState();
	}

	function cancelAllConfigs() {
		applicationConfig.value.isDirty = false;
		instanceConfig.value.isDirty = false;

		applicationConfig.value.isEditing = false;
		instanceConfig.value.isEditing = false;

		// Fetch the backend state to update the store
		void fetchBackendState();
	}

	async function setExecutables(executables: InstanceExecutable[]) {
		if (!instanceConfig.value.config) {
			throw new Error('Instance config is not initialized yet!');
		}

		await taurpc.instances.set_executables(executables);

		// instanceConfig.value.config.executables = executables;
		// instanceConfig.value.isDirty = true;

		// saveInstanceConfig();
	}

	const instanceVariables = computed(() => {
		if (!instanceConfig.value.config) {
			return [];
		}

		const removeTrailingSlash = (str: string) => {
			return str.endsWith('/') ? str.slice(0, -1) : str;
		};

		return [
			{
				key: 'instance',
				value: removeTrailingSlash(instanceConfig.value.config.paths.root)
			},
			{
				key: 'game',
				value: removeTrailingSlash(instanceConfig.value.config.paths.game)
			}
		];
	});

	const replaceInstanceVariables = (str: string) => {
		let tempStr = str;

		for (const variable of instanceVariables.value) {
			tempStr = tempStr.replace(`$${variable.key}`, variable.value);
		}

		return tempStr;
	};

	// -------------------
	// Handle IPC Events
	// -------------------
	listen('ipc', (rawEvent: { event: string; payload: IPCPayload }) => {
		console.log('IPC event:', rawEvent);
		if (rawEvent.event !== 'ipc') {
			return;
		}

		const { payload } = rawEvent;

		switch (payload.command) {
			case 'nxm': {
				if (payload.args.length < 1) {
					console.error('NXM: Failed to handle nxm link, invalid arguments');
					break;
				}

				const nxmLink = `${payload.args[0]}`.replaceAll('"', '');
				taurpc.downloads.download_urls([nxmLink]);

				break;
			}
		}
	});

	return {
		// Application State
		applicationState,
		fetchBackendState,
		// Application Config
		applicationConfig,
		saveApplicationConfig,
		// Frontend Config
		getFrontendConfig,
		updateFrontendConfig,
		// Instance Config
		instanceConfig,
		saveInstanceConfig,
		saveAllConfigs,
		cancelAllConfigs,
		setExecutables,
		// Instance Variables
		instanceVariables,
		replaceInstanceVariables
	};
});
