<template>
	<div>
		<InstancePathConfig v-model="store.instanceConfig.config!.paths" />

		<v-divider class="my-4"></v-divider>

		<VFSConfig v-model="store.instanceConfig.config!.vfs_config" is-nullable />
	</div>
</template>

<script setup lang="ts">
import { taurpc } from '@/lib/taurpc';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
import VFSConfig from '@/components/vfs/VFSConfig.vue';
import InstancePathConfig from './InstancePathConfig.vue';
const store = useApplicationStateStore();
</script>

<style scoped>

</style>