<template>
	<div>
		<v-chip
				v-if="displayValue[0]"
				color="success"
				label
			>
			${{ displayValue[0] }}
		</v-chip>
		{{ displayValue[1] }}
	</div>
</template>

<script setup lang="ts">
// import { open } from '@tauri-apps/api/dialog';
// import { taurpc } from '@/lib/taurpc';
import { computed } from 'vue';

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

const props = defineProps({
	value: {
		type: null as unknown as PropType<string | null>,
		default: null
	},
	variables: {
		type: undefined as unknown as PropType<{ key: string; value: string }[] | undefined>,
		default: undefined
	},
	useInstanceVariables: {
		type: Boolean,
		default: true
	}
});
const emit = defineEmits(['update:modelValue']);

const combinedVariables = computed(() => {
	return [...(props.variables ?? []), ...(props.useInstanceVariables ? store.instanceVariables : [])];
});

const displayValue = computed(() => {
	if (typeof props.value !== 'string' || !combinedVariables.value) return [];

	let foundVariable: string | undefined = undefined;

	// Check if value starts with $
	if (props.value.startsWith('$')) {
		for (const variable of combinedVariables.value) {
			if (props.value.substring(1).startsWith(variable.key)) {
				foundVariable = variable.key;
				break;
			}
		}
	}

	return [
		foundVariable,
		foundVariable ? props.value.substring(foundVariable.length + 1) : props.value
	];
});
</script>

<style scoped>

</style>