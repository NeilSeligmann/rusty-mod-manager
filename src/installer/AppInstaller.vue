<template>
	<div>
		<div v-if="isInitialized">
			<!-- Valid: {{ store.fomod?.isValid() }} <br />
			Invalid Reason: {{ store.fomod?.reasonForInvalidity() ?? 'None' }} <br /> -->
			
			<!-- <img
				v-if="store.fomod?.moduleImage"
				:src="store.getImageUrl(store.fomod.moduleImage)"
				class="mx-auto mb-4 max-w-screen-sm"
				> -->
			
			<Fomod v-if="fomodStore.fomod" :fomod="fomodStore.fomod" />
			<FilePicker v-else />
		</div>
		<div v-else class="p-4 text-center">
			<div v-if="stage !== 'extracting'" class="line-through text-gray-500">
				<v-icon icon="mdi-check-circle-outline" size="small"></v-icon>
				<span>Extracted</span>
				<br />
			</div>
			<div class="pl-4">
				<span>{{ statusText }}</span>
			</div>
			<v-progress-linear indeterminate></v-progress-linear>
		</div>
	</div>
</template>

<style lang="scss" scoped>
@tailwind base;
@tailwind components;
@tailwind utilities;
</style>

<script setup lang="ts">
import { emit, once } from '@tauri-apps/api/event';
import type { InstallerPayload } from '@/lib/bindings';
import { getCurrent } from '@tauri-apps/api/window';
import { taurpc } from '@/lib/taurpc';
import Fomod from './components/Fomod.vue';
import FilePicker from './components/FilePicker.vue';

import { useInstallerStore } from './stores/InstallerStore';
const store = useInstallerStore();

import { useFomodStore } from './stores/FomodStore';
const fomodStore = useFomodStore();

const isInitialized = ref(false);
const stage = ref<'extracting' | 'intializing' | 'done'>('extracting');
// const extractedRelativePath = ref<string | null>(null);

getCurrent().setTitle('Installer');

once('installer-data', async data => {
	// Debounce event
	if (store.installerData) return;

	console.log('Got installer data', data);
	store.installerData = JSON.parse(data.payload as string) as unknown as InstallerPayload;
	getCurrent().setTitle(`Installer - ${store.installerData.file_name}`);

	// Extract archive
	stage.value = 'extracting';
	const extractedPath = await taurpc.downloads.extract_file(store.installerData);
	stage.value = 'intializing';

	await store.initialize(extractedPath);
	// console.log('Initialized installer!')
	stage.value = 'done';
	// store.setExtractedPath(extractedPath);

	// if (store.fomod?.moduleName) {
	// 	getCurrent().setTitle(`Installer - ${store.fomod?.moduleName}`);
	// }

	isInitialized.value = true;
});

emit('ready');

const statusText = computed(() => {
	if (stage.value === 'extracting') return 'Extracting archive...';
	if (stage.value === 'intializing') return 'Processing the archive...';
	if (!isInitialized.value) return 'Initializing...';

	return 'Ready';
});
</script>