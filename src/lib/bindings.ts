 // This file has been generated by Specta. DO NOT EDIT.

export type ApplicationConfig = { available_instances_paths?: string[]; nexusmods: NexusModsConfig; default_vfs_config?: VFSConfig }

export type ApplicationState = { application_config?: ApplicationConfig; frontend_config?: FrontendConfig; selected_instance_path?: string | null; selected_instance?: GameInstance | null; instances_errors?: InstanceError[]; is_vfs_mounted?: boolean; running_executables_id?: { [key in string]: number[] } }

export type AvailableInstancesResponse = { instances: GameInstance[]; errors: InstanceError[] }

export type BethesdaPlugin = { description: string | null; override_record_count: string; file_name: string | null; header_version: number | null; is_light_plugin: boolean; is_master_file: boolean; is_valid_as_light_plugin: boolean; masters: string[]; record_and_group_count: number | null; overlaps_with: string[] | null }

export type CaseFoldingConfig = { enabled?: boolean }

export type Download = { file_name: string; status: DownloadStatus; size_total: string; size_downloaded: string; url: string; md5: string | null; error: string | null; added_at?: string; completed_at: string | null; nexus_data: DownloadNexusData | null }

export type DownloadNexusData = { mod_id: string; file_id: string }

export type DownloadStatus = "Queued" | "Downloading" | "Merging" | "Downloaded" | "Verifying" | "Failed"

export type DownloadsConfig = { concurrent_downloads?: string; threads_per_download?: string }

export type FileStructureSegment = { segment: string; isFile: boolean; children: FileStructureSegment[] | null }

export type FoundSteamGame = { absolute_path: string; steam_game: SupportedSteamGame }

export type FrontendConfig = { sidebar_pinned: boolean }

export type GameIdentifier = "Generic" | "Oblivion" | "Morrowind" | "Skyrim" | "SkyrimSE" | "Fallout3" | "FalloutNV" | "Fallout4"

export type GameInstance = { config: GameInstanceConfig; mods?: InstanceMod[]; mods_indexes?: { [key in string]: number }; mods_errors?: { [key in string]: string }; downloads?: Download[] }

export type GameInstanceConfig = { name: string; steam_id?: string | null; paths: GameInstancePaths; vfs_config?: VFSConfig | null; executables?: InstanceExecutable[]; game_identifier?: GameIdentifier; folding_config?: CaseFoldingConfig; downloads_config?: DownloadsConfig }

export type GameInstanceDeploymentPaths = { mods: string; settings: string | null; saves: string | null }

export type GameInstanceInternalPaths = { mods: string; downloads: string; settings: string; saves: string }

export type GameInstancePaths = { root: string; game: string; internal: GameInstanceInternalPaths; deployment: GameInstanceDeploymentPaths }

export type IPCPayload = { command: string; args: string[] }

export type InstallMod = { name: string; version: string; info: ModInfo; files: InstallModFile[] }

export type InstallModFile = { source: string; destination: string }

export type InstallerPayload = { file_name: string; absolute_path: string; is_relative: boolean }

export type InstanceError = { error: string; instance_path: string }

export type InstanceExecutable = { path: string | null; command: string | null; args: string | null; icon: string | null; name: string; show_shortcut: boolean | null; use_compability?: boolean; use_proton_tricks?: boolean }

export type InstanceMod = { name: string; versions: string[]; selected_version_identifier: string; enabled: boolean; info: ModInfo }

export type ModInfo = { author: string | null; website: string | null; description: string | null; categories: string[] }

export type NMCDNOptionsResponse = { name: string; short_name: string; URI: string }

export type NMDownloadUrl = { url: string; filename: string; md5: string | null; file_request: NMSchemeParameters }

export type NMSchemeParameters = { game_domain: string; mod_id: string; file_id: string; key: string | null; expires: string | null }

export type NexusModsConfig = { api_key?: string | null; user_data?: NexusModsValidateResponse | null; rate_limit?: RateLimit }

export type NexusModsValidateResponse = { user_id: number; key: string; name: string; email: string; profile_url: string; is_premium: boolean; is_supporter: boolean }

export type RateLimit = { hourly_limit?: number | null; hourly_remaining?: number | null; hourly_reset_timestamp?: string | null; daily_limit?: number | null; daily_remaining?: number | null; daily_reset_timestamp?: string | null }

export type SupportedSteamGame = { app_id: number; public_name: string; known_binaries: InstanceExecutable[]; paths: SupportedSteamGamePaths; game_identifier: GameIdentifier | null }

export type SupportedSteamGamePaths = { mods_root: string; profiles_root: string; saves_root: string }

export type TauRpcApiDownloadsInputTypes = { proc_name: "download_urls"; input_type: { __taurpc_type: string[] } } | { proc_name: "resume_downloads"; input_type: null } | { proc_name: "delete_downloads"; input_type: { __taurpc_type: string[] } } | { proc_name: "open_download_in_filemanager"; input_type: { __taurpc_type: string } } | { proc_name: "open_extracted_folder"; input_type: { __taurpc_type: string } } | { proc_name: "install_file"; input_type: { __taurpc_type: string } } | { proc_name: "extract_file"; input_type: { __taurpc_type: InstallerPayload } } | { proc_name: "list_extracted_path_flattened"; input_type: { __taurpc_type: string } } | { proc_name: "list_file_structure_relatively"; input_type: { __taurpc_type: string } } | { proc_name: "read_extracted_file"; input_type: [string, string] } | { proc_name: "install_mod_from_extracted"; input_type: [string, InstallMod] } | { proc_name: "on_downloads_update"; input_type: { __taurpc_type: Download[] } }

export type TauRpcApiDownloadsOutputTypes = { proc_name: "download_urls"; output_type: null } | { proc_name: "resume_downloads"; output_type: null } | { proc_name: "delete_downloads"; output_type: null } | { proc_name: "open_download_in_filemanager"; output_type: null } | { proc_name: "open_extracted_folder"; output_type: null } | { proc_name: "install_file"; output_type: null } | { proc_name: "extract_file"; output_type: UnpackedFileResponse } | { proc_name: "list_extracted_path_flattened"; output_type: string[] } | { proc_name: "list_file_structure_relatively"; output_type: FileStructureSegment[] } | { proc_name: "read_extracted_file"; output_type: number[] } | { proc_name: "install_mod_from_extracted"; output_type: null } | { proc_name: "on_downloads_update"; output_type: null }

export type TauRpcApiInputTypes = { proc_name: "get_state"; input_type: { __taurpc_type: boolean } } | { proc_name: "on_state_changed"; input_type: { __taurpc_type: ApplicationState } } | { proc_name: "get_config_path"; input_type: null } | { proc_name: "update_application_config"; input_type: { __taurpc_type: ApplicationConfig } } | { proc_name: "update_frontend_config"; input_type: { __taurpc_type: FrontendConfig } } | { proc_name: "open_folder"; input_type: { __taurpc_type: string } } | { proc_name: "show_file_in_filemanager"; input_type: { __taurpc_type: string } }

export type TauRpcApiInstancesInputTypes = { proc_name: "create_simple"; input_type: [string, GameInstancePaths] } | { proc_name: "select"; input_type: { __taurpc_type: string } } | { proc_name: "deselect"; input_type: null } | { proc_name: "list_available_instances"; input_type: null } | { proc_name: "update_config"; input_type: { __taurpc_type: GameInstanceConfig } } | { proc_name: "create_empty_mod"; input_type: { __taurpc_type: string } } | { proc_name: "reload_mods"; input_type: null } | { proc_name: "open_mod_folder"; input_type: { __taurpc_type: string } } | { proc_name: "move_mod_by_index"; input_type: [number, number] } | { proc_name: "move_mods_by_indexes"; input_type: [number[], number] } | { proc_name: "move_mod_by_name"; input_type: [string, number] } | { proc_name: "delete_mod_version"; input_type: [string, string | null] } | { proc_name: "delete_mod"; input_type: { __taurpc_type: string } } | { proc_name: "set_mod_enabled"; input_type: [string, boolean] } | { proc_name: "set_mod_active_version"; input_type: [string, string] } | { proc_name: "set_executables"; input_type: { __taurpc_type: InstanceExecutable[] } } | { proc_name: "run_executable"; input_type: { __taurpc_type: InstanceExecutable } } | { proc_name: "stop_executable"; input_type: { __taurpc_type: InstanceExecutable } } | { proc_name: "get_plugins"; input_type: null } | { proc_name: "mount_vfs"; input_type: null } | { proc_name: "unmount_vfs"; input_type: null }

export type TauRpcApiInstancesOutputTypes = { proc_name: "create_simple"; output_type: GameInstance } | { proc_name: "select"; output_type: GameInstance } | { proc_name: "deselect"; output_type: null } | { proc_name: "list_available_instances"; output_type: AvailableInstancesResponse } | { proc_name: "update_config"; output_type: null } | { proc_name: "create_empty_mod"; output_type: InstanceMod } | { proc_name: "reload_mods"; output_type: null } | { proc_name: "open_mod_folder"; output_type: null } | { proc_name: "move_mod_by_index"; output_type: null } | { proc_name: "move_mods_by_indexes"; output_type: number[] } | { proc_name: "move_mod_by_name"; output_type: null } | { proc_name: "delete_mod_version"; output_type: null } | { proc_name: "delete_mod"; output_type: null } | { proc_name: "set_mod_enabled"; output_type: null } | { proc_name: "set_mod_active_version"; output_type: null } | { proc_name: "set_executables"; output_type: null } | { proc_name: "run_executable"; output_type: null } | { proc_name: "stop_executable"; output_type: null } | { proc_name: "get_plugins"; output_type: { [key in string]: BethesdaPlugin[] } } | { proc_name: "mount_vfs"; output_type: null } | { proc_name: "unmount_vfs"; output_type: null }

export type TauRpcApiNexusModsInputTypes = { proc_name: "validate_user"; input_type: null }

export type TauRpcApiNexusModsOutputTypes = { proc_name: "validate_user"; output_type: null }

export type TauRpcApiOutputTypes = { proc_name: "get_state"; output_type: ApplicationState } | { proc_name: "on_state_changed"; output_type: null } | { proc_name: "get_config_path"; output_type: string } | { proc_name: "update_application_config"; output_type: boolean } | { proc_name: "update_frontend_config"; output_type: boolean } | { proc_name: "open_folder"; output_type: null } | { proc_name: "show_file_in_filemanager"; output_type: null }

export type UnpackedFileResponse = { relative_folder: string; absolute_folder: string }

export type VFSConfig = { implementation?: VFSImplementation; command?: string | null }

export type VFSImplementation = "UnionFSFuse" | "OverlayFS"

export type VFSMountConfig = { mount_name: string; command: string | null; paths: VFSMountPaths }

export type VFSMountPaths = { target: string; sources: string[]; overwrite: string; workdir: string }

const ARGS_MAP = {"instances":"{\"list_available_instances\":[],\"stop_executable\":[\"executable\"],\"get_plugins\":[],\"set_executables\":[\"executables\"],\"open_mod_folder\":[\"mod_name\"],\"unmount_vfs\":[],\"reload_mods\":[],\"set_mod_active_version\":[\"mod_name\",\"mod_version\"],\"delete_mod_version\":[\"mod_name\",\"mod_version\"],\"delete_mod\":[\"mod_name\"],\"run_executable\":[\"executable\"],\"create_simple\":[\"name\",\"paths\"],\"create_empty_mod\":[\"name\"],\"set_mod_enabled\":[\"mod_name\",\"enabled\"],\"mount_vfs\":[],\"select\":[\"path\"],\"deselect\":[],\"move_mod_by_name\":[\"mod_name\",\"target_index\"],\"move_mods_by_indexes\":[\"indexes\",\"target_index\"],\"update_config\":[\"config\"],\"move_mod_by_index\":[\"mod_index\",\"target_index\"]}","downloads":"{\"resume_downloads\":[],\"list_file_structure_relatively\":[\"extracted_file\"],\"delete_downloads\":[\"filenames\"],\"extract_file\":[\"filename\"],\"open_download_in_filemanager\":[\"filename\"],\"open_extracted_folder\":[\"extracted_file\"],\"read_extracted_file\":[\"extracted_file\",\"paths\"],\"install_mod_from_extracted\":[\"extracted_file\",\"install_mod\"],\"list_extracted_path_flattened\":[\"extracted_file\"],\"on_downloads_update\":[\"downloads\"],\"install_file\":[\"filename\"],\"download_urls\":[\"url\"]}","":"{\"on_state_changed\":[\"new_state\"],\"open_folder\":[\"path\"],\"show_file_in_filemanager\":[\"path\"],\"get_state\":[\"with_downloads\"],\"get_config_path\":[],\"update_application_config\":[\"config\"],\"update_frontend_config\":[\"config\"]}","nexusmods":"{\"validate_user\":[]}"}
import { createTauRPCProxy as createProxy } from "taurpc"

export const createTauRPCProxy = () => createProxy<Router>(ARGS_MAP)

type Router = {
	'': [TauRpcApiInputTypes, TauRpcApiOutputTypes],
	'instances': [TauRpcApiInstancesInputTypes, TauRpcApiInstancesOutputTypes],
	'nexusmods': [TauRpcApiNexusModsInputTypes, TauRpcApiNexusModsOutputTypes],
	'downloads': [TauRpcApiDownloadsInputTypes, TauRpcApiDownloadsOutputTypes],
}