<template>
	<div>
		<v-tabs
			v-model="tab"
			bg-color="purple"
			height="64"
			fixed-tabs
			>
			<v-tab value="instance">Instance</v-tab>
			<v-tab value="application">Global</v-tab>
		</v-tabs>

		<div class="w-full h-full px-2">
			<v-tabs-window v-model="tab" class="h-full overflow-y-auto">
				<!-- Per Instance settings -->
				<v-tabs-window-item value="instance">
					<InstanceConfig />
					<!-- <InstanceConfig v-model="instanceConfig" /> -->
				</v-tabs-window-item>

				<!-- Global/Application Settings -->
				<v-tabs-window-item value="application">
					<GlobalConfig />
				</v-tabs-window-item>
			</v-tabs-window>
		</div>

		<!-- Save/Cancel button -->
		<v-slide-y-reverse-transition>
			<div v-if="store.applicationConfig.isDirty || store.instanceConfig.isDirty" class="fixed bottom-8 right-0 bg-red-300 w-72 flex flex-row justify-end">
				<!-- Cancel button -->
				<v-fab
					size="large"
					color="error"
					extended
					prepend-icon="mdi-cancel"
					text="Cancel"
					@click="cancelConfig"
				/>

				<!-- Save button -->
				<v-fab
					size="large"
					color="success"
					extended
					prepend-icon="mdi-content-save"
					text="Save"
					@click="saveConfig"
				/>
			</div>
		</v-slide-y-reverse-transition>
	</div>
</template>

<script setup lang="ts">
import { definePage } from 'vue-router/auto';
import { taurpc } from '@/lib/taurpc';
import InstanceConfig from '@/components/settings/InstanceConfigTab.vue';
import GlobalConfig from '@/components/settings/GlobalConfigTab.vue';
import type { ApplicationConfig, GameInstanceConfig } from '@/lib/bindings';

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

definePage({
	meta: {
		title: 'Settings'
	}
});

const tab = ref<string>('instance');

// const instanceConfig = ref<GameInstanceConfig>();

// function getConfigFromStore() {
// 	instanceConfig.value = store.instanceConfig.config;
// }

async function saveConfig() {
	try {
		await store.saveAllConfigs();
	} catch (error) {
		console.error('Failed to save configuration');
		console.error(error);
	}
}

function cancelConfig() {
	store.cancelAllConfigs();
}
</script>

<style scoped></style>
