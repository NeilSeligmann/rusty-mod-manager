<template>
	<div class="pa-2">
		<!-- Instance Paths -->
		<h2 class="text-xl">Instance Paths</h2>
		<span class="text-sm text-gray-300">
			These are the paths for the instance itself. Where to save mods, downloads, saves, and settings.
		</span>

		<div class="pl-2 mt-2">
			<!-- Instance Root -->
			<FolderInput v-model="model.root" label="Instance Root" :read-only="true"/>

			<!-- Instance Paths -->
			<FolderInput
				v-model="model.internal.mods"
				:variables="pathVariables"
				label="Mods Path"
				/>
			<FolderInput
				v-model="model.internal.downloads"
				:variables="pathVariables"
				label="Downloads Path"
				/>
			<FolderInput
				v-model="model.internal.saves"
				:variables="pathVariables"
				label="Saves Path"
				/>
			<FolderInput
				v-model="model.internal.settings"
				:variables="pathVariables"
				label="Settings Path"
				/>

			</div>

		<v-divider class="py-2"></v-divider>

		<h2 class="text-xl">Deployment Paths</h2>

		<div class="pl-2 mt-2">
			<!-- Game Path -->
			<FolderInput v-model="model.game" label="Game Path" />

			<!-- Deployment -->
			<FolderInput v-model="model.deployment.mods" :variables="pathVariables" label="Deployment - Mods" />
			<FolderInput v-model="model.deployment.saves" :variables="pathVariables" label="Deployment - Saves" />
			<FolderInput v-model="model.deployment.settings" :variables="pathVariables" label="Deployment - Settings" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { taurpc } from '@/lib/taurpc';
import FolderInput from '../inputs/FolderInput.vue';
import type { GameInstancePaths } from '@/lib/bindings';
// import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
// const store = useApplicationStateStore();

const model = defineModel<GameInstancePaths>({ required: true });

const pathVariables = computed(() => {
	const variables = [
		{
			key: 'instance',
			value: model.value.root
		},
		{
			key: 'game',
			value: model.value.game
		}
	];

	return variables;
});
</script>

<style scoped>

</style>