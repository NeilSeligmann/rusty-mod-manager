<template>
	<div>
		<!-- Nexus Mods -->
		<NexusModsConfig />

		<v-divider class="my-4"></v-divider>

		<!-- VFS Config -->
		<VFSConfig v-model="store.applicationConfig.config.default_vfs_config" is-default />
	</div>
</template>

<script setup lang="ts">
import { taurpc } from '@/lib/taurpc';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
import NexusModsConfig from '@/components/settings/NexusModsConfig.vue';
import VFSConfig from '@/components/vfs/VFSConfig.vue';
const store = useApplicationStateStore();
</script>

<style scoped>

</style>