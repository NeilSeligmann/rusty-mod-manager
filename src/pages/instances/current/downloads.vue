<template>
	<div class="h-full">
		<SelectableTable
			v-model:sort-by="sortBy"
			:model-value="items"
			:headers="headers"
			item-key="name"
			:allow-dragging="false"
			:context-menu="tableContextMenu"
		>
			<!-- Toolbar -->
			<template #toolbar>
				<v-toolbar flat color="orange-darken-3">
					<!-- Title -->
					<v-toolbar-title>Downloads</v-toolbar-title>

					<v-text-field
						v-model="searchText"
						label="Search"
						prepend-inner-icon="mdi-magnify"
						variant="outlined"
						hide-details
						single-line
					></v-text-field>

					<!-- Spacer -->
					<v-spacer></v-spacer>

					<!-- Add button -->
					<v-btn
						variant="outlined"
						prepend-icon="mdi-plus"
						class="mx-2"
						@click="addDownloadDialog = true"
					>
						Add
					</v-btn>

					<!-- three dots menu -->
					<!-- <v-menu>
						<template #activator="{ props }">
							<v-btn icon="mdi-dots-vertical" class="ml-2" v-bind="props"></v-btn>
						</template>

						<v-list>
							<v-list-item class="cursor-pointer">
								<v-list-item-title>Todo</v-list-item-title>
							</v-list-item>
						</v-list>
					</v-menu> -->
				</v-toolbar>
			</template>

			<!-- No mods warning -->
			<template #no-data>
				<v-icon icon="mdi-dow" size="x-large" /> <br />
				Your downloads are empty.<br />
				Go download something!
			</template>

			<!-- Column - Progress -->
			<template #column-progress="{ item }">
				<v-progress-linear
					height="20"
					:color="item.progress === 1 ? 'success' : 'primary'"
					:indeterminate="item.progress === 0"
					:model-value="item.progress * 100"
				>
					<strong>{{ itemProgressDisplay(item) }}</strong>
				</v-progress-linear>
			</template>

			<!-- Column - Size -->
			<template #column-size="{ item }">
				{{ formatBytes(item.size) }}
			</template>

			<!-- Column - Mod Id -->
			<template #column-modId="{ item }">
				{{ item?.modId || '' }}
			</template>

			<!-- Column - Added At -->
			<template #column-addedAt="{ item }">
				<DateDisplay :timestamp="item.addedAt!" />
			</template>

			<!-- Column - Completed At -->
			<template #column-completedAt="{ item }">
				<DateDisplay v-if="item.completedAt" :timestamp="item.completedAt!" />
			</template>
		</SelectableTable>

		<v-dialog
			v-model="addDownloadDialog"
			max-width="600">
			<v-card
				prepend-icon="mdi-download-multiple"
				title="Add downloads">
				<!-- Card content -->
				<v-card-text>
					<v-textarea
						v-model="downloadUrls"
						placeholder="Enter URLs to download, one per line."
						rows="5"
						hide-details
					/>
				</v-card-text>

				<!-- Card actions -->
				<v-card-actions>
					<v-spacer></v-spacer>
					<v-btn
						@click="addDownloadDialog = false; downloadUrls = '';">
						Cancel
					</v-btn>
					<v-btn
						color="primary"
						prepend-icon="mdi-download-multiple"
						@click="addDownload">
						Add
					</v-btn>
				</v-card-actions>
			</v-card>
		</v-dialog>
	</div>
</template>

<script setup lang="ts">
import { definePage } from 'vue-router/auto';
import { taurpc } from '@/lib/taurpc';
import type { Download, DownloadStatus } from '@/lib/bindings';
import DateDisplay from '@/components/display/DateDisplay.vue';
import SelectableTable from '@/components/selectableTable/SelectableTable.vue';
import type { IContextMenu } from '@/components/selectableTable/SelectableTable.vue';

// @ts-expect-error
import type { SortItem } from 'vuetify/lib/components/VDataIterator/index.mjs';

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

definePage({
	meta: {
		title: 'Downloads'
	}
});

const sortBy: Ref<SortItem> = ref([{ key: 'addedAt', order: 'desc' }]);

const searchText = ref('');

const headers = [
	{ title: 'Name', value: 'name', sortable: true },
	// { title: 'URL', value: 'url', sortable: true },
	{ title: 'Progress', value: 'progress', sortable: true },
	{ title: 'Size', value: 'size', sortable: true },
	{ title: 'Mod ID', value: 'modId', sortable: true },
	{ title: 'Added At', value: 'addedAt', sortable: true },
	{ title: 'Completed At', value: 'completedAt', sortable: true }
	// { title: 'Actions', value: 'actions', sortable: false }
];

function formatBytes(bytes: number, decimals = 2) {
	if (!+bytes) return '0 Bytes';

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];

	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return `${Number.parseFloat((bytes / k ** i).toFixed(dm))} ${sizes[i]}`;
}

const items = computed(() => {
	const downloads = store.applicationState.selected_instance?.downloads ?? [];

	return downloads.map((download: Download) => {
		const downloaded = Number.parseInt(download.size_downloaded, 10);
		const totalSize = Number.parseInt(download.size_total, 10);

		return {
			name: download.file_name,
			progress: downloaded / totalSize,
			size: totalSize,
			status: download.status,
			addedAt: download.added_at,
			completedAt: download.completed_at,
			modId: download.nexus_data?.mod_id,
			_item: download
		};
	});
});

function itemProgressDisplay(item: { progress: number; status: DownloadStatus }) {
	switch (item.status) {
		case 'Downloading':
			return `${Math.ceil(item.progress * 100)}%`;
		case 'Merging':
			return 'Merging...';
		case 'Verifying':
			return 'Verifying...';
		case 'Downloaded':
			return 'Finished';
		default:
			return 'Unknown';
	}
}

const tableContextMenu = computed((): IContextMenu => {
	return {
		items: [
			{
				label: 'Open in NexusMods',
				icon: 'mdi mdi-open-in-new',
				condition: (item: Download) => item.nexus_data !== undefined,
				onClick: (item: Download) => {
					// TODO: Add support for other games
					taurpc.open_file_or_url(`https://www.nexusmods.com/skyrimspecialedition/mods/${item.nexus_data!.mod_id}`);
				},
			},
			{
				label: 'Show in Folder',
				icon: 'mdi mdi-folder-open-outline',
				onClick: (item: Download) => {
					taurpc.downloads.open_download_in_filemanager(item.file_name);
				},
			},
			{
				label: 'Delete Download',
				icon: 'mdi mdi-delete-outline',
				onClick: (item: Download) => {
					taurpc.downloads.delete_downloads([item.file_name]);
				},
			},
		]
	}
})

// Add download dialog
const addDownloadDialog = ref(false);
const downloadUrls = ref('');
function addDownload() {
	// taurpc.downloads.download_url('http://localhost:13373/babo.7z');
	taurpc.downloads.download_urls(downloadUrls.value.split('\n').filter(Boolean));
	downloadUrls.value = '';
	addDownloadDialog.value = false
}

function handleRowDoubleClick(event: MouseEvent, item: any) {
	console.log('Row double clicked', item.item.name);
	// taurpc.downloads.open_download_in_filemanager(item.item.name);
	taurpc.downloads.install_file(item.item.name);
}
</script>

<style scoped></style>
