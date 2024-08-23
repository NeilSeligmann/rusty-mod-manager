<template>
	<div class="pa-4">
		<VAlert
			density="compact"
			title="The file picker has not been fully implemented yet!"
			text="You can still rename/add/delete files using your file explorer."
			type="warning"
			/>

		<v-container>
			<v-row class="">
				<v-col cols="8">
					<VCombobox
						v-model="installMod.name"
						:items="nameItems"
						label="Name (required)"
						/>
				</v-col>
				<v-col cols="4">
					<VTextField
						v-model="installMod.version"
						label="Version"
						placeholder="1.7.2"
						/>
				</v-col>
			</v-row>

			<v-row class="my-0">
				<v-col cols="6">
					<VTextField
						v-model="installMod.info.author"
						label="Author"
						/>
				</v-col>
				<v-col cols="6">
					<VTextField
						v-model="installMod.info.website"
						label="Website"
						/>
				</v-col>
			</v-row>

			<!-- TODO: Add categories -->
		</v-container>

		<!-- Actions -->
		<div class="mb-6 w-full flex justify-center gap-3">
			<VBtn
				prepend-icon="mdi-refresh"
				variant="outlined"
				size="large"
				@click="store.listFileStructure"
				>
				Refresh
			</VBtn>
			<VBtn
				prepend-icon="mdi-folder-open-outline"
				color="success"
				variant="tonal"
				size="large"
				@click="openFolder"
				>
				Open Folder
			</VBtn>
			<VBtn
				prepend-icon="mdi-download"
				color="primary"
				size="large"
				@click="finalizeInstallation"
				>
				Install
			</VBtn>
		</div>

		<VTreeview
			v-model:selected="selectedFiles"
			title="Files to install"
			:items="items"
			slim
			:selectable="false"
			:activatable="false"
			open-strategy="multiple"
			select-strategy="classic"
			variant="flat"
			open-all
			:return-object="false"
			>
		</VTreeview>
	</div>
</template>

<script setup lang="ts">
import { VTreeview } from 'vuetify/labs/VTreeview'
import { useInstallerStore } from '@/installer/stores/InstallerStore';
import { useFomodStore } from '../stores/FomodStore';
import { taurpc } from '@/lib/taurpc';

import type { FileStructureSegment, InstallMod } from '@/lib/bindings';
const store = useInstallerStore();
const fomodStore = useFomodStore();

const selectedFiles = ref([]);
// const openedFiles = ref([]);

const items = computed(() => {
	const processItem = (item: FileStructureSegment): { id: string; title: string; children: any } => {
		return {
			id: item.segment,
			title: item.segment,
			children: item.children?.map(processItem)
		}
	};

	return store.fileStructure.map(processItem);
});

const nameItems = computed(() => {
	const removeEndingFiletype = (name: string) => {
		const splitted = name.split('.');
		if (splitted.length > 1) {
			return splitted.slice(0, -1).join('.');
		}

		return name;
	}

	const names = [
		removeEndingFiletype(store.unpackedPaths!.relative_folder.split('-')[0].split('_unpacked')[0]),
		removeEndingFiletype(store.unpackedPaths!.relative_folder.split('_unpacked')[0]),
		store.unpackedPaths!.relative_folder.split('-')[0].split('_unpacked')[0],
		store.unpackedPaths!.relative_folder.split('_unpacked')[0]
	];

	if (fomodStore.fomodInfo?.data.Name) {
		names.unshift(fomodStore.fomodInfo.data.Name);
	}

	// Remove duplicates
	const uniqueNames = new Set(names);

	return Array.from(uniqueNames);
});

const installMod = ref<InstallMod>({
	name: nameItems.value[0],
	version: fomodStore.fomodInfo?.data.Version ?? '1.0.0',
	info: {
		author: fomodStore.fomodInfo?.data.Author ?? null,
		website: fomodStore.fomodInfo?.data.Website ?? null,
		categories: fomodStore.fomodInfo?.data.Groups ?? [],
		description: null
	},
	files: []
})

const openFolder = () => {
	taurpc.downloads.open_extracted_folder(store.unpackedPaths!.relative_folder);
};

const finalizeInstallation = () => {
	installMod.value.files = store.fileStructureFlattened.map(file => ({
		source: file,
		destination: file
	}));
	store.finalizeInstallation(false, installMod.value);
}
</script>