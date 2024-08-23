<template>
	<v-text-field
		v-model="proxyValue"
		:append-icon="readOnly ? undefined : 'mdi-file-search-outline'"
		:append-inner-icon="proxyValue ? `mdi-file-eye-outline` : ''"
		:label="label"
		:hint="hint"
		:persistent-hint="!!hint"
		variant="outlined"
		:disabled="disabled"
		:readonly="readOnly"
		@click:append-inner="showInFolder"
		@click:append="openFilePicker"
		@keydown.backspace="handleBackspace"
		@blur="onBlur"
	>
		<template #prepend-inner>
			<v-chip
				v-if="variableKey"
				color="success"
				label
			>
			{{ variableKey }}
		</v-chip>
		</template>
	</v-text-field>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog';
import { taurpc } from '@/lib/taurpc';
import { computed } from 'vue';

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();

const props = defineProps({
	modelValue: {
		type: null as unknown as PropType<string | null>,
		default: null
	},
	label: {
		type: undefined as unknown as PropType<string | undefined>,
		default: undefined
	},
	hint: {
		type: undefined as unknown as PropType<string | undefined>,
		default: undefined
	},
	originalValue: {
		type: undefined as unknown as PropType<string | undefined>,
		default: undefined
	},
	variables: {
		type: undefined as unknown as PropType<{ key: string; value: string }[] | undefined>,
		default: undefined
	},
	useInstanceVariables: {
		type: Boolean,
		default: true
	},
	disabled: {
		type: undefined as unknown as PropType<boolean | undefined>,
		default: false
	},
	readOnly: {
		type: undefined as unknown as PropType<boolean | undefined>,
		default: false
	}
});
const emit = defineEmits(['update:modelValue']);

const combinedVariables = computed(() => {
	return [...(props.variables ?? []), ...(props.useInstanceVariables ? store.instanceVariables : [])];
});

const variableKey = computed(() => {
	if (typeof props.modelValue !== 'string' || !combinedVariables.value) return undefined;

	let foundVariable: string | undefined = undefined;

	// Check if value starts with $
	if (props.modelValue.startsWith('$')) {
		for (const variable of combinedVariables.value) {
			if (props.modelValue.substring(1).startsWith(variable.key)) {
				foundVariable = variable.key;
				break;
			}
		}
	}

	if (!foundVariable) return undefined;

	return `$${foundVariable}`;
});

const proxyValue = computed({
	get() {
		if (variableKey.value) {
			return props.modelValue?.substring(variableKey.value.length) ?? null;
		}
		return props.modelValue;
	},
	set(newValue: string | null) {
		if (variableKey.value) {
			newValue = `${variableKey.value}${newValue}`;
		}

		emit('update:modelValue', newValue);
	}
});

const valueWithReplacedVariables = computed(() => {
	if (!combinedVariables.value || !props.modelValue) return props.modelValue;

	let value = `${props.modelValue}`;

	for (const variable of combinedVariables.value) {
		value = value.replace(`$${variable.key}`, variable.value);
	}

	return value;
});

async function openFilePicker() {
	if (props.disabled || props.readOnly) return;

	const selected = await open({
		directory: false,
		multiple: false
	});

	if (!selected) return;

	emit('update:modelValue', selected);
}

async function showInFolder() {
	if (!valueWithReplacedVariables.value) return;
	taurpc.show_file_in_filemanager(valueWithReplacedVariables.value);
}

function handleBackspace(event: KeyboardEvent) {
	if (!props.modelValue || !variableKey.value) return;
	if ((proxyValue.value?.length ?? 0) > 0) return;

	// Prevent the text input from also processing the backspace
	event.preventDefault();

	emit('update:modelValue', `${props.modelValue.substring(0, props.modelValue.length - 1)}`);
}

// Ensure that the path always starts with a slash, if we are using a variable
// TODO: This would be better handled in the backend / rust
function onBlur() {
	if (!props.modelValue || !variableKey.value) return;
	if (proxyValue.value && proxyValue.value.length > 0) return;

	proxyValue.value = '/';
}
</script>

<style scoped>

</style>