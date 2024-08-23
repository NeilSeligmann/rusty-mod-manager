import { defineStore } from 'pinia';
import { getCurrent } from '@tauri-apps/api/window';
import { taurpc } from '@/lib/taurpc';
import { parseInfoDoc, parseModuleDoc, FomodInfo } from 'fomod/src';
// import Fomod from '../utils/Fomod';
import { useFomodStore } from './FomodStore';
import type { InstallerPayload, InstallMod, FileStructureSegment, UnpackedFileResponse } from '@/lib/bindings';

export const useInstallerStore = defineStore('installerStore', () => {
	const installerData = ref<InstallerPayload | null>(null);
	const unpackedPaths = ref<UnpackedFileResponse | null>(null);
	const fileStructure = ref<FileStructureSegment[]>([]);
	const fileStructureFlattened = ref<string[]>([]);
	const fomodFilepath = ref<string | null>(null);
	const infoFilepath = ref<string | null>(null);
	const relativeRootPath = ref<string>('/');
	const imagesUrls = ref<Map<string, string>>(new Map());
	const isInitializing = ref<boolean>(false);

	const initialize = async (path: UnpackedFileResponse) => {
		if (isInitializing.value) {
			return;
		}
		isInitializing.value = true;

		unpackedPaths.value = path;

		// setExtractedPath(path);
		await listFileStructureFlattened();
		await listFileStructure();
		if (findFomodFiles()) {
			console.log('Found fomod files', fomodFilepath.value, infoFilepath.value);

			// We have a valid fomod
			// await preloadImages();
		}

		await loadFomodInfo();
		await loadFomod();

		const fomodStore = useFomodStore();
		fomodStore.intialize();

		console.log('Installer Store intialized!');
	};

	// const setExtractedPath = (path: string) => {
	// 	extractedRelativePath.value = path;
	// };

	const listFileStructure = async () => {
		const strucutre = await taurpc.downloads.list_file_structure_relatively(unpackedPaths.value!.relative_folder);

		fileStructure.value = strucutre;
	};

	const listFileStructureFlattened = async () => {
		const strucutre = await taurpc.downloads.list_extracted_path_flattened(unpackedPaths.value!.relative_folder);

		fileStructureFlattened.value = strucutre;
	};

	const findFomodFiles = () => {
		for (const file of fileStructureFlattened.value) {
			if (file.toLowerCase().endsWith('fomod/moduleconfig.xml')) {
				fomodFilepath.value = file;
				// Set relative root path
				relativeRootPath.value = `${fomodFilepath.value.substring(
					0,
					fomodFilepath.value.lastIndexOf('/fomod')
				)}/`;
			} else if (file.toLowerCase().endsWith('fomod/info.xml')) {
				infoFilepath.value = file;
			}

			if (fomodFilepath.value && infoFilepath.value) {
				return true;
			}
		}

		return false;
	};

	const loadFomodInfo = async () => {
		if (!infoFilepath.value) {
			return null;
		}

		const infoBytes = new Uint8Array(
			await taurpc.downloads.read_extracted_file(unpackedPaths.value!.relative_folder, infoFilepath.value)
		);
		const infoString = new TextDecoder('utf-16le').decode(infoBytes).toString();

		const fomodStore = useFomodStore();
		fomodStore.setDataStrings(undefined, infoString);
	};

	const loadFomod = async () => {
		if (!fomodFilepath.value) {
			return null;
		}

		// Read fomod file
		const fomodBytes = await taurpc.downloads.read_extracted_file(
			unpackedPaths.value!.relative_folder,
			fomodFilepath.value
		);
		// Convert bytes to Uint8Array
		const bytesAsU8 = new Uint8Array(fomodBytes);
		// Convert btyes to string UTF-16 LE
		const fomodString = new TextDecoder('utf-16le').decode(bytesAsU8).toString();

		const fomodStore = useFomodStore();
		fomodStore.setDataStrings(fomodString, undefined);
	};

	// const createBlobUrlFromPath = async (path: string) => {
	// 	// Fetch binary file data
	// 	console.log('Requesting file', path);
	// 	const bytes = await taurpc.downloads.read_extracted_file(extractedRelativePath.value, path);
	// 	console.log('File recieved', path);
	// 	const bytesAsU8 = new Uint8Array(bytes);

	// 	const blob = new Blob([bytesAsU8], { type: 'application/octet-binary' });

	// 	return URL.createObjectURL(blob);
	// };

	// const preloadImages = async () => {
	// 	console.log('Preloading images...');
	// 	console.time('imagesPreload');
	// 	const imagesToPreload = fileStructureFlattened.value.filter(
	// 		filePath => filePath.toLowerCase().endsWith('.png') || filePath.toLowerCase().endsWith('.jpg')
	// 	);

	// 	console.log('imagesToPreload', imagesToPreload);

	// 	await Promise.all(
	// 		imagesToPreload.map(async filePath => {
	// 			if (imagesUrls.value.has(filePath)) {
	// 				return;
	// 			}

	// 			const blobUrl = await createBlobUrlFromPath(filePath);
	// 			imagesUrls.value.set(filePath, blobUrl);
	// 			console.log('Preloaded image', filePath, blobUrl);
	// 		})
	// 	);

	// 	console.log('finished preloading images!');
	// 	console.timeEnd('imagesPreload');
	// };

	const getImageUrl = (filePath: string) => {
		// if (isRelative) {
		filePath = `${relativeRootPath.value}${filePath.split('\\').join('/')}`;
		// }

		if (!installerData.value?.file_name) {
			throw new Error('No installer data');
		}

		return `reqimg://localhost/?basefolder=${encodeURI(unpackedPaths.value!.absolute_folder)}&filename=${encodeURI(
			filePath
		)}`;

		// return imagesUrls.value.get(filePath);
	};

	const convertRelativeFilePathToAbsolute = (filePath: string) => {
		return `${relativeRootPath.value}${filePath
			.split('\\')
			.filter(segment => segment.length > 0)
			.join('/')}`;
	};

	const finalizeInstallation = async (useFomod: boolean, installMod?: InstallMod) => {
		if (useFomod) {
			const fomodStore = useFomodStore();

			if (!installerData.value) {
				throw new Error('No installer data');
			}

			const installMod: InstallMod = {
				name: fomodStore.fomodInfo?.data.Name ?? installerData.value.file_name,
				version: fomodStore.fomodInfo?.data.Version ?? '1.0.0',
				info: {
					author: fomodStore.fomodInfo?.data.Author ?? null,
					categories: fomodStore.fomodInfo?.data.Groups ?? [],
					website: fomodStore.fomodInfo?.data.Website ?? null,
					description: null
				},
				files: fomodStore.flattenedSelectedFiles.map(file => {
					return {
						source: convertRelativeFilePathToAbsolute(file.fileSource),
						destination: file.fileDestination ?? file.fileSource
					};
				})
			};

			await taurpc.downloads.install_mod_from_extracted(unpackedPaths.value!.relative_folder, installMod);
		} else {
			if (!installMod) {
				throw new Error('No installMod provided!');
			}

			await taurpc.downloads.install_mod_from_extracted(unpackedPaths.value!.relative_folder, installMod);
		}

		// Close window
		getCurrent().close();
	};

	return {
		installerData,
		initialize,
		fomodFilepath,
		relativeRootPath,
		getImageUrl,
		fileStructure,
		fileStructureFlattened,
		imagesUrls,
		finalizeInstallation,
		listFileStructure,
		unpackedPaths
	};
});
