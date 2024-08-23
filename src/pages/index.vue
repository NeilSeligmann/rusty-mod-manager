<template>
	<div>
		Index! <br />
		{{ store.applicationState }}
	</div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { taurpc } from '../lib/taurpc';

import { useApplicationStateStore } from '@/stores/ApplicationStateStore';
const store = useApplicationStateStore();
</script>

<style scoped></style>
