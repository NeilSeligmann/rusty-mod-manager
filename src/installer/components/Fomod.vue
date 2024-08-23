<template>
	<div class="p-2">
		<v-row>
			<v-col cols="6">
				<img
					v-if="fomodStore.fomod?.moduleImage"
					:src="imageToShow"
					class="mx-auto mb-4 w-full max-w-screen-sm"
					>
				<div class="bg-black">
					{{ descriptionToShow }}
				</div>
			</v-col>
			<v-col cols="6" class="pr-4">
				<FomodStep @hover="onHover" />
			</v-col>
		</v-row>
		<v-row no-gutters justify="end" class="gap-3">
			<VBtn
				prepend-icon="mdi-arrow-left"
				:disabled="fomodStore.totalSteps === 0 || fomodStore.currentStepIndex === 0"
				@click="fomodStore.moveStep(false)">
				Previous
			</VBtn>
			<VBtn
				:prepend-icon="isLastStep ? 'mdi-check' : 'mdi-arrow-right'"
				color="primary"
				:disabled="!fomodStore.canContinue"
				@click="increaseStep">
				{{ isLastStep ? 'Finish' : 'Next' }}
			</VBtn>
		</v-row>

		<div>
			{{ fomodStore.fomodInfo }}
		</div>
		<div>
			<div v-for="(file, index) in fomodStore.flattenedSelectedFiles" :key="index">
				{{ file.fileSource }} -> {{ file.fileDestination }}
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import FomodStep from './FomodStep.vue';

import { useInstallerStore } from '@/installer/stores/InstallerStore';
const store = useInstallerStore();

import { useFomodStore } from '../stores/FomodStore';
const fomodStore = useFomodStore();

const imageToShow = ref<string | undefined>();
const defaultImage = ref<string | undefined>();
if (fomodStore.fomod?.moduleImage) {
	defaultImage.value = store.getImageUrl(fomodStore.fomod?.moduleImage)
}

const descriptionToShow = ref<string | undefined>();
const defaultDescription = ref<string | undefined>();
// if (fomodStore.fomod?.moduleImage) {
// 	defaultImage.value = store.getImageUrl(fomodStore.fomod?.moduleImage)
// }

const onHover = (image?: string, description?: string) => {
	if (!image) {
		imageToShow.value = defaultImage.value;
	} else {
		imageToShow.value = store.getImageUrl(image);
	}

	if (!description) {
		descriptionToShow.value = defaultDescription.value;
	} else {
		descriptionToShow.value = description;
	}
}

const isLastStep = computed(() => {
	return fomodStore.currentStepIndex >= fomodStore.totalSteps - 1;
});

const increaseStep = () => {
	if (isLastStep.value) {
		store.finalizeInstallation(true);
		return;
	}

	fomodStore.moveStep(true);
}
</script>