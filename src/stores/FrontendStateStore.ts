import { defineStore } from 'pinia';
import { Ref, ref } from 'vue';

export const useFrontendStateStore = defineStore('frontendState', () => {
	const bAutomaticallRedirectToCurrentInstance = ref(true);

	function shouldRedirectToCurrentInstance() {
		if (bAutomaticallRedirectToCurrentInstance.value) {
			bAutomaticallRedirectToCurrentInstance.value = false;
			return true;
		}

		return false;
	}

	return { shouldRedirectToCurrentInstance };
});
