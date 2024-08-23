<template>
	<v-card
		:subtitle="props.isDefault ? `Configure the default way mods will be deployed` : `Configure how the mods will be deployed`">

		<template #title>
			Virtual File System (VFS)
			<v-chip v-if="props.isDefault" size="small" class="ml-2">Default</v-chip>
		</template>

		<v-card-text>
			<!-- Select implementation -->
			<v-select
				v-model="selectedItem"
				:items="selectItems"
				label="File System Implementation"
				:hint="(props.isNullable && selectedItem === 'Default') ? `The default implementation is: ${store.applicationConfig.config.default_vfs_config!.implementation!}` : undefined"
				:persistent-hint="props.isNullable && selectedItem === 'Default'"
				:required="!isNullable"
			/>
			
			<!-- Custom command -->
			<!-- TODO: Custom Command -->
		</v-card-text>
	</v-card>
</template>

<script setup lang="ts">
import type { VFSConfig, VFSImplementation } from '@/lib/bindings';
const model = defineModel<VFSConfig | undefined | null>({ required: true });
const props = defineProps<{
	isNullable?: boolean;
	isDefault?: boolean;
}>();
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

const selectItems = ref<(VFSImplementation | 'Default')[]>(['UnionFSFuse', 'OverlayFS']);
const selectedItem = ref<VFSImplementation | 'Default'>('Default');

if (props.isNullable) {
	selectItems.value.unshift('Default');
}

watch(() => selectedItem.value, updateModel);

function updateModel() {
	// If none, set implementation to undefined
	if (selectedItem.value === 'Default') {
		model.value = undefined;
		return;
	}

	// Initialize model if not set
	if (!model.value) {
		model.value = {
			implementation: selectedItem.value,
			command: undefined
		};
	} else {
		model.value.implementation = selectedItem.value;
	}
}

watch(() => model.value, updateSelected);

function updateSelected() {
	if (!model.value) {
		selectedItem.value = 'Default';
		return;
	}

	selectedItem.value = model.value.implementation || 'Default';
}
updateSelected();
</script>

<style scoped>

</style>