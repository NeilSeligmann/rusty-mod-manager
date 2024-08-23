<template>
	<v-layout style="height: 100%">
		<v-navigation-drawer
			:expand-on-hover="!store.getFrontendConfig('sidebar_pinned')"
			:rail="!store.getFrontendConfig('sidebar_pinned')"
			:rail-width="60"
			:mobile="false"
			:width="220"
			:permanent="true"
		>
			<div class="flex flex-col h-full">
				<div style="display: flex; width: 200px" class="flex flex-row justify-start">
					<v-btn
						flat
						variant="text"
						:icon="store.getFrontendConfig('sidebar_pinned') ? 'mdi-menu-open' : 'mdi-menu-close'"
						size="large"
						title="Pin sidebar"
						@click="togglePinSidebar"
					/>
				</div>

				<!-- Instance Side-bar -->
				<v-list>
					<v-list-item
						prepend-icon="mdi-folder"
						title="Mods"
						:to="{ path: '/instances/current/mods' }"
						value="mods"
						color="primary"
					></v-list-item>
					<v-list-item
						prepend-icon="mdi-application-export"
						title="Exectuables"
						:to="{ path: '/instances/current/executables' }"
						value="executables"
						color="green-darken-1"
					></v-list-item>
					<v-list-item
						prepend-icon="mdi-download"
						title="Downloads"
						:to="{ path: '/instances/current/downloads' }"
						value="downloads"
						color="orange-darken-2"
					></v-list-item>
					<v-list-item
						prepend-icon="mdi-cog"
						title="Settings"
						:to="{ path: '/instances/current/settings' }"
						value="settings"
						color="purple-lighten-1"
					></v-list-item>
				</v-list>

				<v-divider />
				<v-list>
					<v-list-item
						prepend-icon="mdi-swap-horizontal"
						title="Change Instance"
						value="mods"
						color="primary"
						:active="false"
						@click="exitInstance"
					></v-list-item>
				</v-list>

				<v-divider class="mb-auto" />

				<!-- VFS Indicator -->
				<div class="w-full text-center p-2">
					<!-- <v-btn variant="outlined" class="w-full" size="x-large" color="grey">
						VFS Inactive
					</v-btn> -->
					<v-card
						:color="isVfsActive ? 'primary' : 'grey'"
						:variant="isVfsActive ? 'tonal' : 'outlined'"
						ripple
						hover
						:loading="isVfsLoading"
						:disabled="isVfsLoading"
						@click="toggleVfs">
						<v-card-text class="w-full">
							<div class="flex justify-center align-middle">
								<v-icon
								:icon="isVfsActive ? `mdi-folder` : `mdi mdi-folder-off-outline`"
								size="large"
								class="mr-1"
								/>
								<p class="pt-0.5">
									{{ isVfsActive ? 'VFS Mounted' : 'VFS Inactive'}}
								</p>
							</div>
							<!-- <p class="text-sm">
								{{ isVfsActive ? `(Click to unmount)` : `(Click to mount)` }}
							</p> -->
						</v-card-text>
					</v-card>
				</div>
			</div>
		</v-navigation-drawer>

		<v-main>
			<!-- Drag and Drop indicator -->
			<div v-if="isDraggingFile" class="z-[9000] absolute top-0 left-0 w-full h-full flex flex-col text-center justify-center align-middle bg-gray-950 bg-opacity-80 text-white">
				<div class="border border-dotted mx-auto w-52 py-8 bg-slate-800 rounded-sm">
					<v-icon icon="mdi-upload" size="x-large" color="white" />
					<p class="text-white text-lg">Drop file here</p>
					<span class="text-sm text-gray-400">Only supports archives</span>
				</div>
			</div>

			<!-- Router View -->
			<router-view v-slot="{ Component }">
				<keep-alive>
					<component :is="Component" />
				</keep-alive>
			</router-view>
		</v-main>
	</v-layout>
</template>

<script setup lang="ts">
import router from '@/plugins/router';
import { GameInstance } from '@/lib/bindings';
import { taurpc } from '@/lib/taurpc';
import { listen } from '@tauri-apps/api/event'

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();
const currInstance = store.applicationState.selected_instance;

onMounted(() => {
	if (!currInstance) {
		router.push({ path: '/instances' });
		return;
	}

	if (router.currentRoute.value.path === '/instances/current') {
		router.push({ path: '/instances/current/mods' });
	}
});

function togglePinSidebar() {
	store.updateFrontendConfig('sidebar_pinned', !store.getFrontendConfig('sidebar_pinned'));
}

const isVfsLoading = ref(false);

const isVfsActive = computed(() => {
	return store.applicationState.is_vfs_mounted ?? false;
});

async function toggleVfs() {
	if (isVfsLoading.value) {
		return;
	}

	isVfsLoading.value = true;

	try {
		if (isVfsActive.value) {
			await taurpc.instances.unmount_vfs();
		} else {
			await taurpc.instances.mount_vfs();
		}
	} catch (error) {
		console.error('Failed to mount/unmount VFS:');
		console.error(error);
	}

	isVfsLoading.value = false;
}

async function exitInstance() {
	await taurpc.instances.deselect();
	router.push({ path: '/instances' });
}

const isDraggingFile = ref(false);

// Listen for drag and drop
listen('tauri://file-drop', (event: { event: string, payload: string[] }) => {
	console.log('file-dropped', event)
	taurpc.downloads.install_file(event.payload[0]);
})

listen('tauri://file-drop-hover', event => {
	isDraggingFile.value = true;
	console.log('file-hover', event)
})

listen('tauri://file-drop-cancelled', event => {
	isDraggingFile.value = false;
	console.log('file-cancelled', event)
})
</script>

<style scoped></style>
