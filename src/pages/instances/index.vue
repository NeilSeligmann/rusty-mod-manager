<template>
	<div>
		<v-data-table :items="items" :items-per-page="50" :headers="headers" :sort-by="[{ key: 'name' }]">
			<template #top>
				<v-toolbar flat color="primary">
					<v-toolbar-title>Instances</v-toolbar-title>
					<v-spacer></v-spacer>
					<v-btn variant="outlined" prepend-icon="mdi-plus" size="large" @click="newInstance">
						New Instance
					</v-btn>
				</v-toolbar>
			</template>

			<template v-slot:body.prepend>
				<tr
					v-for="error in applicationStore.applicationState.instances_errors"
					class="bg-red-800"
					@click="openInstanceFolder(error.instance_path)"
				>
					<td class="mr-2 font-bold"colspan="2">
						<v-icon class="ml-1 mr-1">mdi-alert</v-icon>
						{{ error.instance_path }}
					</td>
					<td colspan="2">
						{{ error.error }}
					</td>
				</tr>
			</template>

			<!-- eslint-disable-next-line vue/v-slot-style vue/valid-v-slot -->
			<template v-slot:item.actions="props">
				<v-btn
					:color="isInstanceSelected(props.item.config.paths.root) ? 'success' : 'primary'"
					prepend-icon="mdi-exit-to-app"
					@click="selectInstance(props.item.config.paths.root)"
				>
					{{ isInstanceSelected(props.item.config.paths.root) ? 'Current' : 'Select' }}
				</v-btn>
				<!-- <v-btn
					class="ml-2"
					color="warning"
					prepend-icon="mdi-pencil"
					@click="editInstance(props.item.config.paths.root)"
				>
					Edit
				</v-btn> -->
			</template>
		</v-data-table>

		<v-dialog v-model:model-value="showNewInstanceDialog" max-width="600">
			<NewInstanceDialog />
		</v-dialog>
	</div>
</template>

<script setup lang="ts">
import router from '@/plugins/router';
import type { GameInstance } from '@/lib/bindings';
import { taurpc } from '@/lib/taurpc';
import NewInstanceDialog, { type NewInstanceData } from '@/components/instance/NewInstanceDialog.vue';

// Stores
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
import { useFrontendStateStore } from '@/stores/FrontendStateStore';
const applicationStore = useApplicationStateStore();
const frontendStore = useFrontendStateStore();

const headers = [
	{ title: 'Instance Name', value: 'config.name', sortable: true },
	{ title: 'Path', value: 'config.paths.root', sortable: true },
	{ title: 'Game', value: 'config.game_identifier', sortable: true },
	{ title: 'Actions', value: 'actions' }
];

const items = ref<GameInstance[]>([]);

taurpc.instances.list_available_instances().then(availableInstances => {
	items.value = availableInstances.instances;

	// Automatically navigate to selected instance, if any
	// TODO: Check if this is the initial boot of the app
	// if not, then don't navigate automatically
	if (frontendStore.shouldRedirectToCurrentInstance() && applicationStore.applicationState.selected_instance_path) {
		router.push({ path: '/instances/current' });
	}
});

function isInstanceSelected(path: string) {
	return path && applicationStore.applicationState.selected_instance_path === path;
}

async function selectInstance(path: string) {
	await taurpc.instances.select(path);
	router.push({ path: '/instances/current' });
}

// const newInstanceDialog = ref<{ show: boolean, data: NewInstanceData | null }>({
// 	show: false,
// 	data: null
// });
const showNewInstanceDialog = ref(false);

function newInstance() {
	// newInstanceDialog.value.data = null;
	// newInstanceDialog.value.show = true;
	showNewInstanceDialog.value = true;
}

function openInstanceFolder(path: string) {
	taurpc.open_folder(path);
}
</script>

<style scoped></style>
