<template>
	<div class="w-[42px] flex flex-row justify-center align-middle" @click="startStopExecutable">
		<v-avatar v-if="executable.icon" :title="executable.name" :loading="isLoading" :image="`data:image/png;base64,${executable.icon}`" />
		<v-avatar v-else :title="executable.name" :loading="isLoading" color="grey-darken-3">
			<span>{{ executable.name }}</span>
		</v-avatar>
		
		<VBtn
			:title="executable.name"
			:loading="isLoading"
			:class="{ '!opacity-30': isRunning, '!opacity-100': isLoading }"
			width="42px"
			height="42px"
			class="bg-black opacity-0 hover:!opacity-100"
			:icon="`mdi ${isRunning ? 'mdi-stop' : 'mdi-play'}`"
			variant="tonal"
			position="absolute"
		/>
	</div>
</template>

<script setup lang="ts">
import type { InstanceExecutable } from '@/lib/bindings';
import { taurpc } from '@/lib/taurpc';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

const props = defineProps({
	executable: {
		type: Object as PropType<InstanceExecutable>,
		required: true
	}
});

const isLoading = ref(false);

async function startStopExecutable() {
	isLoading.value = true;

	try {
		if (isRunning.value) {
			await taurpc.instances.stop_executable(props.executable);
		} else {
			await taurpc.instances.run_executable(props.executable);
		}
	} catch (error) {
		console.error('Failed to start/stop executable');
		console.error(error);
	}
	
	isLoading.value = false;
}

const isRunning = computed(() => {
	return (store.applicationState?.running_executables_id?.[props.executable.name]?.length ?? 0) > 0;
})

</script>

<style scoped>

</style>