<template>
	<div>
		<!-- STEP {{ fomodStore.currentStepIndex + 1 }} / {{ fomodStore.fomod?.steps.size }} <br /> -->
		<!-- Name: {{ fomodStore.currentStep.name }} <br /> -->
		<!-- sortingOrder: {{ fomodStore.fomod?.sortingOrder }} <br /> -->
		<!-- visibilityDeps: {{ fomodStore.fomod. }} <br /> -->
		<!-- tagName: {{ fomodStore.fomod?.tagName }} <br /> -->
		<!-- Name: {{ store.fomod?.currentStep.groups }} <br /> -->

		<h3 class="font-bold text-2xl">{{ fomodStore.currentStep.name }}</h3>

		<FomodGroup
			v-for="group in fomodStore.currentStep.groups"
			:step-name="fomodStore.currentStep.name"
			:key="group.name"
			:group="castedGroup(group)"
			@hover="(image, description) => emits('hover', image, description)"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Fomod as BaseFomod, Group } from 'fomod/src';
import FomodGroup from './FomodGroup.vue';
import { useFomodStore } from '../stores/FomodStore';
const fomodStore = useFomodStore();

const emits = defineEmits<(e: 'hover', image?: string, description?: string) => void>();

const castedGroup = (group: any) => {
	return group as unknown as Group<false>
}
</script>