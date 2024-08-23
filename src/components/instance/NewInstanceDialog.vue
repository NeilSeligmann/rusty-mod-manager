<template>
	<!-- <v-form @submit.prevent> -->
	<v-card>
		<v-container>
			<!-- Instance Name -->
			<v-row>
				<v-col cols="12">
					<v-text-field
						v-model="instanceData.name"
						label="Instance Name"
						required
						hide-details
					></v-text-field>
				</v-col>
			</v-row>

			<!-- Instance Location -->
			<v-row>
				<v-col cols="12">
					<FolderInput
						v-model="instanceData.location"
						label="Instance Location"
						required
						:hint="`The instance will be located at ${finalInstancePath()}`"
						persistent-hint
					/>
				</v-col>
			</v-row>

			<!-- Deployment Paths -->
			<!-- Game Path -->
			<v-row>
				<v-col cols="12">
					<FolderInput v-model="instanceData.paths.game" label="Game Path" required hide-details />
				</v-col>
			</v-row>

			<v-divider :thickness="2" style="margin-top: 2rem; margin-bottom: 2rem"></v-divider>

			<!-- Mods Path -->
			<v-row>
				<v-col cols="12">
					<FolderInput
						v-model="instanceData.paths.deployment.mods"
						label="Mods Deployment Path"
						required
						hide-details
					/>
				</v-col>
			</v-row>

			<!-- Saves Path -->
			<v-row>
				<v-col cols="12">
					<FolderInput
						v-model="instanceData.paths.deployment.saves"
						label="Saves Deployment Path"
						hide-details
						clearable
					/>
				</v-col>
			</v-row>

			<!-- Settings Path -->
			<v-row>
				<v-col cols="12">
					<FolderInput
						v-model="instanceData.paths.deployment.settings"
						label="Settings Deployment Path"
						hide-details
						clearable
					/>
				</v-col>
			</v-row>

			<v-btn type="submit" block class="mt-8" size="large" color="success" @click="createInstance()">
				Create Instance
			</v-btn>
		</v-container>
	</v-card>
	<!-- </v-form> -->
</template>

<script setup lang="ts">
import { ref } from 'vue';
import FolderInput from '@/components/inputs/FolderInput.vue';
import type { GameInstancePaths } from '../../lib/bindings';
import { taurpc } from '@/lib/taurpc';
import router from '@/plugins/router';

export interface NewInstanceData {
	name: string;
	location: string;
	paths: GameInstancePaths;
}

const instanceData = ref<NewInstanceData>({
	name: '',
	location: '',
	paths: {
		root: '',
		game: '',
		internal: {
			downloads: '$instance/downloads',
			mods: '$instance/mods',
			saves: '$instance/saves',
			settings: '$instance/settings'
		},
		deployment: {
			mods: '$game',
			settings: null,
			saves: null
		}
	}
});

// Fetch the default instance path
taurpc.get_config_path().then(path => {
	instanceData.value.location = `${path}/instances`;
});

function finalInstancePath() {
	return `${instanceData.value.location}/${instanceData.value.name}`;
}

async function createInstance() {
	const tempPaths = instanceData.value.paths;
	tempPaths.root = finalInstancePath();

	const newInstance = await taurpc.instances.create_simple(instanceData.value.name, tempPaths);
	router.push({ path: '/instances' });
}
</script>