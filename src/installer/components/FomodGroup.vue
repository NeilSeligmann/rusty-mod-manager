<template>
	<div class="flex flex-col w-full my-4 pl-2 border">
		<!-- <div>Group name: {{ groupModel?.name }}</div> -->
		<!-- <div>Group behaviorType: {{ groupModel?.behaviorType }}</div> -->
		<!-- <div>Group sortingOrder: {{ groupModel?.sortingOrder }}</div> -->
		<!-- <div>Group tagName: {{ groupModel?.tagName }}</div> -->

		<!-- 'SelectAny' | 'SelectAll' | 'SelectExactlyOne' | 'SelectAtMostOne' | 'SelectAtLeastOne'; -->

		<v-radio-group
			v-if="groupModel?.behaviorType === 'SelectExactlyOne' || groupModel?.behaviorType === 'SelectAtMostOne'"
			:model-value="radioGroupValue"
			:value-comparator="valueComparator"
			@update:model-value="(value: any) => updateOption(value as Option<false>, true, true)"
			>
				<h4 class="pl-2 font-bold">
					{{ groupModel?.name }}
					<span v-if="isRequired" class="text-red">*</span>
				</h4>
				<v-radio
					v-for="option in groupModel?.options"
					:value="option"
					:value-comparator="valueComparator"
					@mouseover="showHoverData(option.image)"
					@mouseleave="showHoverData(undefined)"
					>
					<template #label>
						<div class="px-1">
							<h5 class="font-bold border-b-2">{{ option.name }}</h5>
							<div v-if="option.description" class="text-caption border-b-[1px] pl-1" v-html="parseDescription(option.description)" />
						</div>
					</template>
				</v-radio>
		</v-radio-group>

		<div v-if="groupModel?.behaviorType === 'SelectAny' || groupModel?.behaviorType === 'SelectAtLeastOne' || groupModel?.behaviorType === 'SelectAll'">
			<h4 class="pl-2 font-bold">{{ groupModel?.name }}</h4>
			<v-checkbox
				v-for="option in groupModel?.options"
				:model-value="isOptionSelected(option)"
				@update:model-value="(value: any) => updateOption(option, value)"
				:label="option.name"
				@mouseover="showHoverData(option.image, option.description)"
				@mouseleave="showHoverData(undefined)"
				>
				<template #label>
					<div class="px-1">
						<h5 class="font-bold border-b-2">
							{{ option.name }}
							<span v-if="isRequired" class="text-red">*</span>
						</h5>
						<div v-if="option.description" class="text-caption border-b-[1px] pl-1" v-html="parseDescription(option.description)" />
					</div>
				</template>
			</v-checkbox>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Fomod as BaseFomod, Group, Option, Step } from 'fomod/src';
// import FomodGroupOption from './FomodGroupOption.vue';
import { parseDescription } from '../utils/DescriptionParser';
import { useFomodStore } from '../stores/FomodStore';
const fomodStore = useFomodStore();

// const groupModel = defineModel<ReturnType <typeof useFomodStore>['currentStep']['groups']>('group');
const props = defineProps<{
	stepName: string
}>();
const groupModel = defineModel<Group<false>>('group');
const emits = defineEmits<(e: 'hover', image?: string, description?: string) => void>();

const showHoverData = (image?: string | null, description?: string) => {
	emits('hover', image ?? undefined, description);
}

const valueComparator = (option1: Option<false>, option2: Option<false>) => {
	if (!option1 || !option2) return false;

	return option1.name === option2.name;
}

const selectedOptions = computed(() => {
	return fomodStore.selectedOptions[props.stepName][groupModel.value!.name];
})

const radioGroupValue = computed(() => {
	return fomodStore.selectedOptions[props.stepName][groupModel.value!.name][0];
});

const isOptionSelected = (option: Option<false>) => {
	// console.log('isOptionSelected', option, !!selectedOptions.value);
	if (!selectedOptions.value) return false;

	for (const selectedOption of selectedOptions.value) {
		if (selectedOption.name === option.name) return true;
	}

	return false;
}

const updateOption = (option: Option<false>, value: boolean, single = false) => {
	// If Select All, we need to select/unselect all at the same time
	// if (groupModel.value?.behaviorType === 'SelectAll') {
	// biome-ignore lint/correctness/noConstantCondition: <explanation>
	// if (true) {
	// 	for (const suboption of groupModel.value?.options ?? []) {
	// 		_updateOption(suboption, value, false);
	// 	}
	// } else {
		return _updateOption(option, value, single);
	// }
}

const _updateOption = (option: Option<false>, value: boolean, single = false) => {
	if (value) {
		if (single) {
			fomodStore.selectedOptions[props.stepName][groupModel.value!.name] = [option];
		} else {
			fomodStore.selectedOptions[props.stepName][groupModel.value!.name].push(option);
		}
	} else {
		fomodStore.selectedOptions[props.stepName][groupModel.value!.name] = fomodStore.selectedOptions[props.stepName][groupModel.value!.name].filter(x => x.name !== option.name);
	}
}

const isRequired = computed(() => {
	return ['SelectAtLeastOne', 'SelectExactlyOne'].includes(groupModel.value!.behaviorType);
})

// const setOptionValue = (option: Option<false>) => {
// 	console.log('setOptionValue', option);
// 	fomodStore.selectedOptions
// }

</script>