<template>
	<v-card
		title="Nexus Mods"
		subtitle="Configure the NexusMods API Key in order to download mods">

		<v-card-text>
			<v-text-field
				v-model="store.applicationConfig.config.nexusmods.api_key"
				label="API Key"
				placeholder="Your API key goes here" />

			<div v-if="store.applicationConfig.config.nexusmods.user_data" class="flex flex-row gap-3">
				<v-list-item
					:prepend-avatar="store.applicationConfig.config.nexusmods.user_data?.profile_url"
					:subtitle="store.applicationConfig.config.nexusmods.user_data?.email"
				>
					<template v-slot:prepend>
						<v-avatar
							size="40"
							class="ml-4"
							:src="store.applicationConfig.config.nexusmods.user_data?.profile_url" />
					</template>
					<template v-slot:title>
						{{ store.applicationConfig.config.nexusmods.user_data?.name }}
						<v-chip v-if="store.applicationConfig.config.nexusmods.user_data?.is_premium" color="rgb(217, 143, 64)" variant="elevated" size="x-small">Premium</v-chip>
					</template>
				</v-list-item>
				<v-list-item
					title="Daily Requests"
					:subtitle="`${store.applicationConfig.config.nexusmods.rate_limit?.daily_remaining} / ${store.applicationConfig.config.nexusmods.rate_limit?.daily_limit}`"
				>
				</v-list-item>
				<v-list-item
					title="Hourly Requests"
					:subtitle="`${store.applicationConfig.config.nexusmods.rate_limit?.hourly_remaining} / ${store.applicationConfig.config.nexusmods.rate_limit?.hourly_limit}`"
				>
				</v-list-item>

				<v-btn
					prepend-icon="mdi-refresh"
					color="primary"
					class="ml-auto mr-4"
					:loading="isLoading"
					@click="validateNexusModsApiKey">
					Reload
				</v-btn>
			</div>
			<div v-else  class="flex flex-col justify-center align-middle gap-3">
				<v-alert
					v-if="apiKeyError"
					:value="apiKeyError"
					type="error"
					dense>
					{{ apiKeyError }}
				</v-alert>

				<v-btn
					prepend-icon="mdi-login"
					color="primary"
					@click="validateNexusModsApiKey">
					Validate API Key
				</v-btn>
			</div>
		</v-card-text>
	</v-card>
</template>

<script setup lang="ts">
import { taurpc } from '@/lib/taurpc';
import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();
const apiKeyError = ref<string | null>(null);
const isLoading = ref<boolean>(false);

async function validateNexusModsApiKey() {
	isLoading.value = true;

	try {
		await taurpc.nexusmods.validate_user();
	} catch (error) {
		console.error('Failed to validate Nexus Mods API key');
		console.error(error);

		apiKeyError.value = String(error);
	}

	isLoading.value = false;
}
</script>

<style scoped>

</style>