<template>
	<div class="h-full flex flex-col">
		<!-- Toolbar -->
		<v-toolbar class="h-[64px]" flat color="primary">
			<!-- Title -->
			<v-toolbar-title>Mods</v-toolbar-title>

			<!-- Spacer -->
			<v-spacer></v-spacer>

			<!-- Executables -->
			<div class="mx-2 flex flex-row gap-2">
				<ExecutableIcon v-for="(item, index) in executables" :executable="item" />

				<!-- <v-btn v-if="(executables ?? []).length > 0" density="default" icon="mdi-plus" variant="tonal" size="small" @click="addExecutable"></v-btn>
				<v-btn v-else density="default" prepend-icon="mdi-plus" variant="outlined" @click="addExecutable">Add Executable</v-btn> -->
			</div>
		</v-toolbar>

		<!-- Main Content -->
		<div class="flex flex-row flex-grow">
			<!-- Mods Table -->
			<div class="w-auto flex-grow h-full">
				<v-alert
					v-if="Object.keys(store.applicationState.selected_instance?.mods_errors ?? {}).length > 0"
					class="ma-2"
					title="Invalid mods detected!"
					type="error"
					closable
				>
					<v-list lines="one" density="compact" color="error">
						<v-list-item
							v-for="(error, mod) in store.applicationState.selected_instance?.mods_errors"
							:key="mod"
							:title="mod"
							:subtitle="error"
							prepend-icon="mdi-folder-open-outline"
							@click="openModFolder(mod)"
						></v-list-item>
					</v-list>
				</v-alert>

				<selectable-table
					v-model="items"
					operational-key="loadOrder"
					:headers="headers"
					v-model:sort-by="sortBy"
					item-key="name"
					:allow-dragging="sortBy.length === 0 || (sortBy[0].key === 'loadOrder' && sortBy[0].order === 'asc')"
					:context-menu="tableContextMenu"
					@on-dragged="onDragged"
					@on-double-click="onDoubleClick">
					<!-- No data -->
					<template #no-data>
						<v-icon icon="mdi-folder-alert-outline" size="x-large" /> <br />
						There are no mods in this instance. <br />
						Create a new one or import one!
					</template>
	
					<!-- <template #column-name="{ item }">
						{{ item.name }}
					</template> -->
	
					<template #column-actions="{ item }">
						<v-icon v-if="!isStaticMod(item)" icon="mdi mdi-delete" @click="deleteMod(item)" />
					</template>
	
					<template #column-selected_version_identifier="{ item }">
						<ModVersion v-if="!isStaticMod(item)" :mod="item" />
						<span v-else></span>
					</template>
	
					<template #column-enabled="{ item }">
						<VCheckbox
							:disabled="isStaticMod(item)"
							:model-value="item.enabled"
							@update:model-value="(value) => updateEnabled(item, value)"
							hide-details
							density="compact"
							color="primary"/>
					</template>
	
					<template #column-author="{ item }">
						{{ item.info.author }}
					</template>
				</selectable-table>
			</div>

			<!-- Plugins -->
			<div v-if="canShowPlugins" class="w-96 h-full">
				Plugins!
				<!-- {{ store.applicationState.selected_instance?.plugins }} -->
				<VBtn @click="reloadPlugins">Reload Plugins</VBtn>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { definePage } from 'vue-router/auto';
import { taurpc } from '@/lib/taurpc';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
import router from '@/plugins/router';
import SelectableTable from '@/components/selectableTable/SelectableTable.vue';
import type { IContextMenu } from '@/components/selectableTable/SelectableTable.vue';
import ExecutableIcon from '@/components/display/ExecutableIcon.vue';
import ModVersion from '@/installer/components/ModVersion.vue';

// @ts-expect-error
import type { SortItem } from 'vuetify/lib/components/VDataIterator/index.mjs';
import type { InstanceMod } from '@/lib/bindings';

const store = useApplicationStateStore();

definePage({
	meta: {
		title: 'Mods'
	}
});

const headers = [
	{ title: 'Enabled', value: 'enabled', sortable: true },
	{ title: 'Name', value: 'name', sortable: true },
	{ title: 'Version', value: 'selected_version_identifier', sortable: true },
	{ title: 'Load Order', value: 'loadOrder', sortable: true },
	// { title: 'Author', value: 'author', sortable: true },
	// { title: 'Actions', value: 'actions', sortable: false },
];

const items = computed(() => {
	const mods = store.applicationState.selected_instance?.mods ?? [];

	return mods.map((mod, index) => {
		return {
			...mod,
			loadOrder: index
		};
	});
});

const sortBy: Ref<SortItem> = ref([{ key: 'loadOrder', order: 'asc' }]);

async function createEmptyMod() {
	try {
		await taurpc.instances.create_empty_mod('test');
	} catch (e) {
		console.error(e);
	}
}

async function reloadMods() {
	try {
		await taurpc.instances.reload_mods();
	} catch (e) {
		console.error(e);
	}
}

async function openModFolder(modName: string) {
	try {
		await taurpc.instances.open_mod_folder(modName);
	} catch (e) {
		console.error(e);
	}
}

async function onDoubleClick(item: InstanceMod, index: number) {
	openModFolder(item.name);
}

// const bAddExecutableModal = ref(false);

const executables = computed(() =>
	store.applicationState.selected_instance?.config.executables?.filter(e => e.show_shortcut)
);

function addExecutable() {
	router.push({
		path: '/instances/current/executables'
	});
}

async function onDragged(draggingIndexes: number[], droppedAtIndex: number) {
	try {
		const newIndexes = await taurpc.instances.move_mods_by_indexes(draggingIndexes, droppedAtIndex);

		// Set new selected indexes
		// selectedItems.value = newIndexes;
	} catch (error) {
		console.error('Failed to move mod by index:');
		console.error(typeof error);
		console.error(error);
		console.error(JSON.stringify(error));
	}
}

function deleteMod(mod: InstanceMod) {
	try {
		taurpc.instances.delete_mod(mod.name);
	} catch (e) {
		console.error(e);
	}
}

const canShowPlugins = computed(() => store.applicationState.selected_instance?.config.game_identifier !== 'Generic');

async function reloadPlugins() {
	try {
		const plugins = await taurpc.instances.get_plugins();
		console.log('Plugins:', plugins);
	} catch (e) {
		console.error(e);
	}
}

const isStaticMod = (mod: InstanceMod) => {
	return mod.name === 'base' || mod.name === 'overwrite';
}

async function updateEnabled(mod: InstanceMod, value: boolean) {
	await taurpc.instances.set_mod_enabled(mod.name, value);
}

const tableContextMenu = computed((): IContextMenu => {
	return {
		items: [
			{
				label: 'Open Folder',
				icon: 'mdi mdi-folder-open-outline',
				onClick: (item: InstanceMod) => {
					openModFolder(item.name)
				},
			},
			// {
			// 	label: 'Delete Mod',
			// 	icon: 'mdi mdi-delete-outline',
			// 	onClick: (item: InstanceMod) => {
			// 		console.log('deleteMod', item.name);
			// 	},
			// },
		]
	}
})
</script>

<style scoped></style>
