<template>
	<div>
		<VAutocomplete
			v-if="mod.versions.length > 1"
			:model-value="mod.selected_version_identifier"
			:items="mod.versions"
			@update:model-value="(newVersion: string) => updateModVersion(mod, newVersion)"
			single-line
			density="compact"
			hide-details
			variant="underlined"
			@click="(e: MouseEvent) => { e.stopPropagation(); e.preventDefault(); }"
			/>
		<span v-else>{{ mod.selected_version_identifier }}</span>
	</div>
</template>

<script setup lang="ts">
import type { Fomod as BaseFomod, Group } from 'fomod/src';
import FomodGroup from './FomodGroup.vue';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
import type { InstanceMod } from '@/lib/bindings';
import { taurpc } from '@/lib/taurpc';
const store = useApplicationStateStore();

const props = defineProps<{
	mod: InstanceMod
}>();

function updateModVersion(mod: InstanceMod, newVersion: string) {
	try {
		taurpc.instances.set_mod_active_version(mod.name, newVersion);
	} catch (e) {
		console.error(e);
	}
}
</script>