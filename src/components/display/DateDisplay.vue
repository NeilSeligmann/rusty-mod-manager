<template>
	<div>
		{{ formattedTimestamp }}
	</div>
</template>

<script setup lang="ts">

const props = defineProps({
	timestamp: {
		type: [String, Number],
		required: true
	}
});

const timestampAsDate = computed(() => {
	const castedToNumber = Number(props.timestamp);
	const date = new Date(castedToNumber * 1000);

	return date;
});

const formattedTimestamp = computed(() => {
	const formatter = new Intl.DateTimeFormat('en-US', {
		hour: '2-digit',
		minute: '2-digit',
		day: '2-digit',
		month: '2-digit',
		year: 'numeric'
	});
	const formattedDate = formatter.format(timestampAsDate.value);

	// return `${formattedDate}`;
	return formattedDate;
});

</script>

<style scoped>

</style>