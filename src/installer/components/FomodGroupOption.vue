<template>
	<div class="flex flex-col w-full ml-4">
		<div>Option name: {{ optionModel?.name }}</div>
	</div>
</template>

<script setup lang="ts">
import type { Fomod as BaseFomod, Option } from 'fomod/src';
const optionModel = defineModel<Option<false>>('option');
</script>