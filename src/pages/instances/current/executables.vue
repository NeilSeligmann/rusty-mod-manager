<template>
	<div class="h-full">
		<selectable-table
			v-model="executables"
			:headers="headers"
			item-key="name"
			@on-dragged="onDragged">
			<template #toolbar>
				<v-toolbar flat color="green-darken-1">
					<!-- Title -->
					<v-toolbar-title>Executables</v-toolbar-title>
					<!-- <v-divider class="mx-4" inset vertical :thickness="2" ></v-divider> -->
					
					<!-- New Button -->
					<v-btn variant="outlined" prepend-icon="mdi-plus" class="ml-2" @click="addExecutable">
						Add Executable
					</v-btn>

					<!-- Three dots menu -->
					<!-- <v-menu>
						<template #activator="{ props }">
							<v-btn icon="mdi-dots-vertical" class="ml-2" v-bind="props"></v-btn>
						</template>

						<v-list>
							<v-list-item class="cursor-pointer">
								<v-list-item-title>Import</v-list-item-title>
							</v-list-item>
							<v-list-item class="cursor-pointer" @click="reloadMods">
								<v-list-item-title>Reload</v-list-item-title>
							</v-list-item>
						</v-list>
					</v-menu> -->
				</v-toolbar>
			</template>

			<!-- No data -->
			<template #no-data>
				<v-icon icon="mdi-folder-alert-outline" size="x-large" /> <br />
				There are no executables in this instance. <br />
				Create one!
			</template>

			<template #column-icon="{ item }">
				<ExecutableIcon :executable="item" />
			</template>

			<template #column-value="{ item }">
				<span v-if="item.command" class="bg-black px-2 py-1 font-mono">{{ finalCommand(item) }}</span>
				<VariableDisplay v-else :value="item.path" />
			</template>

			<template #column-show_shortcut="{ item }">
				<VSwitch
					color="blue"
					hide-details
					@click="(e: MouseEvent) => e.stopPropagation()" v-model="item.show_shortcut"
					/>
			</template>

			<template #column-actions="{ item, index }">
				<v-btn
					icon="mdi-pencil"
					size="small"
					color="green-darken-1"
					variant="tonal"
					@click="(e: MouseEvent) => { e.stopPropagation(); modifyExecutable(index) }"
					/>
			</template>
		</selectable-table>

		<!-- Add/Modify executable modal -->
		<v-dialog
			v-model="bAddExecutableModal"
			width="auto"
			>
			<v-card
				min-width="400"
				max-width="800"
				prepend-icon="mdi-application-cog-outline"
				:title="executableModifingId !== null ? 'Modify executable' : 'Add an executable'"
			>
				<v-card-text>
					<v-row dense>
						<v-col>
							<v-switch
								v-model="modalExecutableData.show_shortcut"
								label="Show in Hotbar"
								color="green-darken-1"
								/>
						</v-col>
					</v-row>

					<v-row dense>
						<v-col>
							<v-text-field
								label="Name"
								required
								v-model="modalExecutableData.name"
								hint="Name to display"
							/>
						</v-col>
					</v-row>

					<v-divider class="mb-5" />

					<v-row dense>
						<v-col>
							<FileInput
								label="Executable"
								:disabled="!!modalExecutableData.command"
								v-model="modalExecutableData.path"
								:hint="!!modalExecutableData.command ? 'Disabled, will use command below' : ''"
								/>
						</v-col>
					</v-row>

					<v-row dense>
						<v-col>
							<v-text-field
								label="Command"
								required
								v-model="modalExecutableData.command"
								hint="Command to execute"
							/>
						</v-col>
					</v-row>

					<v-row dense>
						<v-col>
							<v-text-field
								label="Arguments"
								required
								placeholder="--somekey=value --anotherkey"
								hint="Arguments passed to the executable"
								v-model="modalExecutableData.args"
							/>
						</v-col>
					</v-row>

					<v-divider class="mb-5" />

					<v-row dense class="flex flex-col justify-center align-middle text-center">
						<p class="mb-2">This executable will run the following command:</p>
						
						<code class="text-sm bg-black font-mono mx-auto px-2 py-1">
							{{ finalCommand(modalExecutableData) }}
						</code>
					</v-row>
				</v-card-text>

				<template v-slot:actions>
					<v-btn
						class="ms-auto"
						text="Cancel"
						@click="bAddExecutableModal = false"
					/>
					<v-btn
						text="Save"
						color="primary"
						@click="saveExecutableModal"
					/>
				</template>
			</v-card>
		</v-dialog>
	</div>
</template>

<script setup lang="ts">
import { definePage } from 'vue-router/auto';
import { taurpc } from '@/lib/taurpc';
import type { Download, DownloadStatus, InstanceExecutable } from '@/lib/bindings';
import SelectableTable from '@/components/selectableTable/SelectableTable.vue';
import FileInput from '@/components/inputs/FileInput.vue';
import VariableDisplay from '@/components/display/VariableDisplay.vue';
import ExecutableIcon from '@/components/display/ExecutableIcon.vue';

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

definePage({
	meta: {
		title: 'Executables'
	}
});

const executables = computed(() => store.applicationState.selected_instance?.config.executables ?? []);

const headers = [
	{ title: 'Name', value: 'name', sortable: false },
	{ title: 'Icon', value: 'icon', sortable: false },
	{ title: 'Value', value: 'value', sortable: false },
	{ title: 'Show in Hotbar', value: 'show_shortcut', sortable: false },
	{ title: 'Actions', value: 'actions', sortable: false }
	// { title: 'Path', value: 'path', sortable: false },
	// { title: 'command', value: 'command', sortable: false },
	// { title: 'args', value: 'args', sortable: false },
	// { title: 'icon', value: 'icon', sortable: false },
];

// ----------------
// Modal
// ----------------

const bAddExecutableModal = ref(false);

const executableModifingId = ref<number | null>(null);
const intialModalExecutableData = () => ({
	name: null,
	path: null,
	command: null,
	args: null,
	icon: null,
	show_shortcut: null
});
const modalExecutableData = ref<InstanceExecutable>(intialModalExecutableData());

function addExecutable() {
	executableModifingId.value = null;
	modalExecutableData.value = intialModalExecutableData();
	bAddExecutableModal.value = true;
}

function modifyExecutable(id: number) {
	executableModifingId.value = id;
	modalExecutableData.value = executables.value[id];
	bAddExecutableModal.value = true;
}

async function saveExecutableModal() {
	const newExecutables: InstanceExecutable[] = executables.value ?? [];

	if (executableModifingId.value === null) {
		newExecutables.push(modalExecutableData.value);
	} else {
		newExecutables[executableModifingId.value] = modalExecutableData.value;
	}

	await store.setExecutables(newExecutables);
	bAddExecutableModal.value = false;
}

watch(modalExecutableData, value => {
	if (value.command === '') value.command = null;
	if (value.path === '') value.path = null;
});

// ----------------
// Other
// ----------------

async function saveExecutables() {
	store.setExecutables(executables.value);
}

function finalCommand(executable: InstanceExecutable) {
	const command =
		typeof executable.command === 'string' && executable.command.length > 0 ? executable.command : executable.path;
	if (!command) return null;

	return store.replaceInstanceVariables(`${command} ${executable.args ?? ''}`);
}

async function onDragged(draggingIndexes: number[], droppedAtIndex: number) {
	try {
		console.log('onDragged:', draggingIndexes, droppedAtIndex);
		// const newIndexes = await taurpc.instances.move_mods_by_indexes(draggingIndexes, droppedAtIndex);

		// Set new selected indexes
		// selectedItems.value = newIndexes;
	} catch (error) {
		console.error('Failed to move mod by index:');
		console.error(typeof error);
		console.error(error);
		console.error(JSON.stringify(error));
	}
}
</script>

<style scoped></style>
