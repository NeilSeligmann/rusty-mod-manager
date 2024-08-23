import { defineStore } from 'pinia';
// import { Ref, ref } from 'vue';
import { taurpc } from '@/lib/taurpc';
import {
	parseInfoDoc,
	parseModuleDoc,
	type FomodInfo,
	type Fomod as BaseFomod,
	type Option,
	type InstallPattern,
	type Install
} from 'fomod/src';

export const useFomodStore = defineStore('fomodStore', () => {
	const fomod = ref<BaseFomod<false> | null>(null);
	const fomodInfo = ref<FomodInfo | null>(null);
	const currentStepIndex = ref<number>(0);

	const fomodString = ref<string | undefined>(undefined);
	const infoString = ref<string | undefined>(undefined);

	function setDataStrings(_fomodString?: string, _infoString?: string) {
		if (typeof fomodString !== 'undefined') {
			fomodString.value = _fomodString;
		}

		if (typeof _infoString !== 'undefined') {
			infoString.value = _infoString;
		}
	}

	function initialize() {
		console.log('Initializing Fomod Store...');
		if (infoString.value) {
			const infoDocument = new DOMParser().parseFromString(infoString.value, 'text/xml');

			const parsedInfo = parseInfoDoc(infoDocument);
			if (parsedInfo) {
				fomodInfo.value = parsedInfo;
			}
		}

		if (fomodString.value) {
			// Parse XML into a document
			const document = new DOMParser().parseFromString(fomodString.value, 'text/xml');

			const parsedModule = parseModuleDoc(document);
			if (!parsedModule) {
				throw new Error('Failed to parse Fomod');
			}
			fomod.value = parsedModule;

			// Initialize every step option vale
			for (const step of allSteps.value) {
				selectedOptions.value[step.name] = {};

				// Initialize every group option value
				for (const group of step.groups) {
					selectedOptions.value[step.name][group.name] = [];
				}
			}
		}

		console.log('Initialized Fomod Store!');
	}

	const moduleData = computed(() => {
		return {
			image: fomod.value?.moduleImage,
			name: fomod.value?.moduleName
		};
	});

	const allSteps = computed(() => {
		return Array.from(fomod.value!.steps);
	});

	const fileteredSteps = computed(() => {
		// TODO: Filter steps
		return allSteps.value.filter(step => {
			// return rray.from(step.visibilityDeps.dependencies.values()).some(dep => dep);
			// Array.from(step.visibilityDeps.dependencies.values()).forEach(dep => {
			// });

			// for (const dep of Array.from(step.visibilityDeps.dependencies)) {
			// 	console.log('dep', dep);
			// 	console.log('dep.operator', dep.operator);
			// 	console.log('dep.dependencies', Array.from(dep.dependencies));
			// }

			return true;
		});
	});

	const currentStep = computed(() => {
		return fileteredSteps.value[currentStepIndex.value];
	});

	const canContinue = computed(() => {
		for (const group of currentStep.value.groups) {
			const isRequired = ['SelectAtLeastOne', 'SelectExactlyOne'].includes(group.behaviorType);
			if (!isRequired) continue;

			if (!selectedOptions.value[currentStep.value.name][group.name].length) {
				return false;
			}
		}

		return true;
	});

	const moveStep = (forward = true) => {
		let targetStepIndex = currentStepIndex.value;

		if (forward) {
			if (!canContinue.value) {
				return;
			}

			targetStepIndex++;
		} else {
			targetStepIndex--;
		}

		if (targetStepIndex < 0) {
			targetStepIndex = 0;
		} else if (targetStepIndex >= fileteredSteps.value.length) {
			targetStepIndex = fileteredSteps.value.length - 1;
		}

		currentStepIndex.value = targetStepIndex;
	};

	const totalSteps = computed(() => {
		return fomod.value?.steps.size || 0;
	});

	// TODO: Filter selected options by filtered steps
	// As we dont want to return no longer valid option
	const selectedOptions = ref<{
		// Steps
		[key: string]: {
			// Groups -> Options[]
			[key: string]: Option<false>[];
		};
	}>({});

	const compiledFlags = computed(() => {
		// This should only return the set flags in the previous steps.
		// Loop every step, in order, until the current one is reached
	});

	const flattenedSelectedOptions = computed(() => {
		const flattenedOptions: Option<false>[] = [];

		for (const stepKey of Object.keys(selectedOptions.value)) {
			for (const groupKey of Object.keys(selectedOptions.value[stepKey])) {
				flattenedOptions.push(...selectedOptions.value[stepKey][groupKey]);
			}
		}

		return flattenedOptions;
	});

	const flattenedSelectedFiles = computed(() => {
		const files: Install<false>[] = [];

		// Set default installs
		if (fomod.value?.installs) {
			const defaultInstallsArray = Array.from(fomod.value.installs) as Install<false>[];
			files.push(...defaultInstallsArray);
		}

		for (const option of flattenedSelectedOptions.value) {
			files.push(...option.installsToSet.filesWrapper.installs);
		}

		// Sort files by priority
		// Higher -> lower - Higher must be installed first
		return files.sort((a, b) => {
			let aPriority = 0;
			let bPriority = 0;

			const parsedA = Number.parseInt(a.priority, 10);
			const parsedB = Number.parseInt(b.priority, 10);

			if (!Number.isNaN(parsedA)) aPriority = parsedA;
			if (!Number.isNaN(parsedB)) bPriority = parsedB;

			return bPriority - aPriority;
		});
	});

	return {
		setDataStrings,
		fomod,
		fomodInfo,
		currentStepIndex,
		intialize: initialize,
		moduleData,
		allSteps,
		fileteredSteps,
		currentStep,
		moveStep,
		totalSteps,
		selectedOptions,
		canContinue,
		flattenedSelectedFiles
	};
});
