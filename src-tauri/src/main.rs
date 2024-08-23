#![allow(non_snake_case)]
#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use controllers::file_controller::{self, FileStructureSegment};
use controllers::plugin_controller::BethesdaPlugin;
use core::panic;
use futures::Future;
use instances::instance_mod::{InstanceMod, ModInfo};
use instances::{GameInstance, GameInstanceConfig, GameInstancePaths, InstanceExecutable};
use mods::downloader::{Download, DownloadNexusData};
use mods::ipc::{self, IPCClient, IPCPayload, IPCServer};
use serde::{Deserialize, Serialize};
use state::{config, default_instances_path, root_config_path, AvailableInstancesResponse};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::SeekFrom;
use std::thread;
use tauri::api::cli::SubcommandMatches;
use tauri::api::file;
use tauri::http::ResponseBuilder;
use url::Url;
// use std::ffi::{OsStr, OsString};
// use instances::IGameInstancePaths;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
// use tauri::api::shell;
// use tauri::SystemTray;
use tauri::{AppHandle, CustomMenuItem, Manager, Menu, Runtime, State, Submenu, Window};
use taurpc::Router;
use tokio::sync::{oneshot, Mutex};

pub mod controllers;
pub mod deployer;
pub mod instances;
pub mod mods;
pub mod state;

pub type MutexState = Arc<Mutex<state::ApplicationState>>;

#[taurpc::ipc_type]
struct InstallModFile {
	source: String,
	destination: String,
}

#[taurpc::ipc_type]
struct InstallMod {
	name: String,
	version: String,
	info: ModInfo,
	files: Vec<InstallModFile>,
}

#[taurpc::ipc_type]
struct InstallerPayload {
	file_name: String,
	absolute_path: String,
	is_relative: bool,
}

#[taurpc::ipc_type]
struct UnpackedFileResponse {
	relative_folder: String,
	absolute_folder: String,
}

// ------------------------------
// Main Interface
// ------------------------------

#[derive(Clone)]
struct ApiImpl {
	state: MutexState,
}

// #[taurpc::procedures(event_trigger = ApiEventTrigger, export_to = "../src/lib/bindings.ts")]
#[taurpc::procedures(event_trigger = ApiEventTrigger, export_to = "../src/lib/bindings.ts")]
trait Api {
	async fn get_state(with_downloads: bool) -> state::ApplicationState;
	#[taurpc(event)]
	async fn on_state_changed(new_state: state::ApplicationState);
	// async fn create_instance(data: instances::GameInstance);
	// async fn with_window<R: Runtime>(window: Window<R>);
	async fn get_config_path() -> PathBuf;
	async fn update_application_config(config: config::ApplicationConfig) -> bool;

	// Frontend config state
	async fn update_frontend_config(config: state::FrontendConfig) -> bool;

	// Utils
	async fn open_folder(path: String) -> Result<(), String>;
	async fn show_file_in_filemanager(path: String) -> Result<(), String>;
}

#[taurpc::resolvers]
impl Api for ApiImpl {
	async fn get_state(self, with_downloads: bool) -> state::ApplicationState {
		let state = self.state.lock().await;
		let mut state_clone = state.clone();

		// Remove downloads if not requested
		if !with_downloads && state_clone.selected_instance.is_some() {
			state_clone
				.selected_instance
				.as_mut()
				.unwrap()
				.downloads
				.clear();
		}

		return state_clone;
	}

	async fn get_config_path(self) -> PathBuf {
		return state::root_config_path();
	}

	async fn update_frontend_config(self, config: state::FrontendConfig) -> bool {
		let mut state = self.state.lock().await;
		state.frontend_config = config;
		state.save().unwrap();
		return true;
	}

	async fn update_application_config(self, config: config::ApplicationConfig) -> bool {
		let mut state = self.state.lock().await;

		// If the API key has changed, validate it
		if state.application_config.nexusmods.api_key != config.nexusmods.api_key {
			state.application_config.nexusmods.api_key = config.nexusmods.api_key.clone();
			state.application_config.nexusmods.user_data = None;

			// Validate API key
			let _ = state.application_config.nexusmods.validate_api_key().await;
			// match response {
			// 	Ok(_) => {}
			// 	Err(e) => {
			// 		tauri::dialog::message(e).run();
			// 	}
			// }
		}

		state.application_config = config;
		state.save().unwrap();
		return true;
	}

	async fn open_folder(self, path: String) -> Result<(), String> {
		return crate::controllers::file_controller::open_folder(PathBuf::from(path));
	}

	async fn show_file_in_filemanager(self, path: String) -> Result<(), String> {
		return crate::controllers::file_controller::open_in_filemanager(PathBuf::from(path));
	}
}

// ------------------------------
// Instances Interface
// ------------------------------

#[derive(Clone)]
struct ApiInstancesStateImpl {
	state: MutexState,
}

// #[taurpc::procedures(event_trigger = ApiEventTrigger, export_to = "../src/lib/bindings.ts")]
#[taurpc::procedures(path = "instances")]
trait ApiInstances {
	async fn create_simple(name: String, paths: GameInstancePaths) -> Result<GameInstance, String>;
	async fn select(path: PathBuf) -> Result<GameInstance, String>;
	async fn deselect() -> Result<(), String>;
	async fn list_available_instances() -> Result<AvailableInstancesResponse, String>;
	async fn update_config(config: GameInstanceConfig) -> Result<(), String>;
	// Mods
	async fn create_empty_mod(name: String) -> Result<InstanceMod, String>;
	async fn reload_mods() -> Result<(), String>;
	async fn open_mod_folder(mod_name: String) -> Result<(), String>;
	async fn move_mod_by_index(mod_index: u32, target_index: u32) -> Result<(), String>;
	async fn move_mods_by_indexes(indexes: Vec<u32>, target_index: u32)
		-> Result<Vec<u32>, String>;
	async fn move_mod_by_name(mod_name: String, target_index: u32) -> Result<(), String>;
	async fn delete_mod_version(
		mod_name: String,
		mod_version: Option<String>,
	) -> Result<(), String>;
	async fn delete_mod(mod_name: String) -> Result<(), String>;
	async fn set_mod_enabled(mod_name: String, enabled: bool) -> Result<(), String>;
	async fn set_mod_active_version(mod_name: String, mod_version: String) -> Result<(), String>;
	// async fn update_vfs_config(vfs_config: Option<config::vfs_config::VFSConfig>) -> Result<(), String>;
	// async fn validate_config(config: GameInstanceConfig) -> Result<(), Vec<String>>;

	// Executables
	async fn set_executables(executables: Vec<InstanceExecutable>) -> Result<(), String>;
	async fn run_executable(executable: InstanceExecutable) -> Result<(), String>;
	async fn stop_executable(executable: InstanceExecutable) -> Result<(), String>;

	// Plugins
	async fn get_plugins() -> Result<HashMap<String, Vec<BethesdaPlugin>>, String>;

	// VFS
	async fn mount_vfs() -> Result<(), String>;
	async fn unmount_vfs() -> Result<(), String>;
}

#[taurpc::resolvers]
impl ApiInstances for ApiInstancesStateImpl {
	// Instance Methods
	async fn create_simple(
		self,
		name: String,
		paths: GameInstancePaths,
	) -> Result<GameInstance, String> {
		let mut state = self.state.lock().await;

		// Check if there already is an instance with the given name
		let existing_instances = state.list_available_instances().await?;
		for instance in existing_instances.instances {
			if instance.config.name == name {
				return Err(format!("Instance with name \"{}\" already exists", name));
			}
		}

		// Check if there already is an instance with the given path
		if instances::GameInstance::exists(paths.root.clone()) {
			return Err(format!(
				"Instance with path \"{}\" already exists",
				paths.root.to_str().unwrap()
			));
		}

		// Create instance
		let new_instance = instances::GameInstance::new(name, paths)?;

		state.add_instance_path(new_instance.config.paths.root.clone())?;

		// Save state
		state.clone().save()?;

		return Ok(new_instance);
	}

	async fn select(self, path: PathBuf) -> Result<GameInstance, String> {
		let mut state = self.state.lock().await;

		// Attempt to load instance
		state.selected_instance = Some(GameInstance::load_from_path(path.clone())?);
		// Set selected instance path
		state.selected_instance_path = Some(path);

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(state.selected_instance.clone().unwrap());
	}

	async fn deselect(self) -> Result<(), String> {
		let mut state = self.state.lock().await;
		state.selected_instance = None;
		state.selected_instance_path = None;

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(());
	}

	async fn list_available_instances(self) -> Result<AvailableInstancesResponse, String> {
		let mut state = self.state.lock().await;
		let list_instances = state.list_available_instances().await;

		return list_instances;
	}

	async fn update_config(self, config: GameInstanceConfig) -> Result<(), String> {
		let mut state = self.state.lock().await;

		match &mut state.selected_instance {
			Some(instance) => {
				instance.update_config(config.clone())?;

				// Update state
				state.trigger_on_state_changed()?;
				return Ok(());
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	// Mods

	async fn create_empty_mod(self, name: String) -> Result<InstanceMod, String> {
		let mut state = self.state.lock().await;

		let selected_instance = state.selected_instance_or_fail();

		// Create mod
		let mod_instance = selected_instance.create_empty_mod(name.into(), None)?;

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(mod_instance);
	}

	async fn reload_mods(self) -> Result<(), String> {
		let mut state = self.state.lock().await;

		match state.selected_instance {
			Some(ref mut instance) => {
				let mods = instance.load_mods()?;

				// Update state
				state.trigger_on_state_changed()?;

				return Ok(());
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	async fn open_mod_folder(self, mod_name: String) -> Result<(), String> {
		let state = self.state.lock().await;

		match &state.selected_instance {
			Some(instance) => {
				instance.open_mod_folder(mod_name)?;
				return Ok(());
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	async fn move_mod_by_index(self, mod_index: u32, target_index: u32) -> Result<(), String> {
		let mut state = self.state.lock().await;

		match &mut state.selected_instance {
			Some(instance) => {
				instance.move_mod_by_index(mod_index, target_index, true)?;

				// Update state
				state.trigger_on_state_changed()?;
				return Ok(());
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	async fn move_mods_by_indexes(
		self,
		indexes: Vec<u32>,
		target_index: u32,
	) -> Result<Vec<u32>, String> {
		let mut state = self.state.lock().await;

		match &mut state.selected_instance {
			Some(instance) => {
				let new_indexes = instance.move_mods_by_indexes(indexes, target_index)?;

				// Update state
				state.trigger_on_state_changed()?;
				return Ok(new_indexes);
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	async fn move_mod_by_name(self, mod_name: String, target_index: u32) -> Result<(), String> {
		let mut state = self.state.lock().await;

		match &mut state.selected_instance {
			Some(instance) => {
				instance.move_mod_by_name(mod_name, target_index)?;

				// Update state
				state.trigger_on_state_changed()?;
				return Ok(());
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	async fn delete_mod(self, mod_name: String) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		selected_instance.delete_mod(mod_name)?;

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(());
	}

	async fn delete_mod_version(
		self,
		mod_name: String,
		mod_version: Option<String>,
	) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		selected_instance.delete_mod_version(mod_name, mod_version)?;

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(());
	}

	async fn set_mod_enabled(self, mod_name: String, enabled: bool) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		// Set active version
		selected_instance.set_mod_enabled(mod_name, enabled)?;

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(());
	}

	async fn set_mod_active_version(
		self,
		mod_name: String,
		mod_version: String,
	) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		// Set active version
		selected_instance.set_mod_active_version(mod_name, mod_version)?;

		// Update state
		state.trigger_on_state_changed()?;

		return Ok(());
	}

	// Executables

	async fn set_executables(self, executables: Vec<InstanceExecutable>) -> Result<(), String> {
		let mut state = self.state.lock().await;

		// let selected_instance = state.selected_instance_or_fail();

		match &mut state.selected_instance {
			Some(instance) => {
				instance.set_executables(executables)?;
				return Ok(());
			}
			None => {
				return Err("No instance selected".to_string());
			}
		}
	}

	async fn run_executable(self, executable: InstanceExecutable) -> Result<(), String> {
		let mut state = self.state.lock().await;

		// Mount VFS, if needed
		if !state.is_vfs_mounted {
			state.mount_vfs()?
		}

		// Run executable
		let selected_instance = state.selected_instance_or_fail();
		let child_process = selected_instance.run_executable(executable.clone())?;

		// Update running executables state
		state.running_executables_add(executable.name, child_process);

		// Save state
		state.save()?;

		return Ok(());
	}

	async fn stop_executable(self, executable: InstanceExecutable) -> Result<(), String> {
		let mut state = self.state.lock().await;

		state.stop_running_executable(executable.name);

		return Ok(());
	}

	// Plugins

	async fn get_plugins(self) -> Result<HashMap<String, Vec<BethesdaPlugin>>, String> {
		let mut state = self.state.lock().await;

		let selected_instance = state.selected_instance_or_fail();

		// Drop state early, as we wont need it later

		let plugins = selected_instance.get_plugins()?;
		// state.trigger_on_state_changed()?;

		selected_instance.get_load_order()?;

		return Ok(plugins);
	}

	// VFS

	async fn mount_vfs(self) -> Result<(), String> {
		let mut state = self.state.lock().await;

		return state.mount_vfs();
	}

	async fn unmount_vfs(self) -> Result<(), String> {
		let mut state = self.state.lock().await;

		return state.unmount_vfs();
	}

	// async fn validate_config(self, config: GameInstanceConfig) -> Result<(), String> {
	// 	let mut state = self.state.lock().await;

	// 	let instance = GameInstance::new(format!("temp"), config.paths.clone())?;
	// 	// Set config
	// 	instance.config = config.clone();

	// 	return Ok(());

	// 	// TODO: Validate instance
	// 	// match instance.validate() {
	// 	// 	Ok(_) => {
	// 	// 		return Ok(());
	// 	// 	}
	// 	// 	Err(e) => {
	// 	// 		return Err(e);
	// 	// 	}
	// 	// }
	// }

	// async fn update_vfs_config(self, vfs_config: Option<config::vfs_config::VFSConfig>) -> Result<(), String> {
	// 	let mut state = self.state.lock().await;

	// 	match &mut state.selected_instance {
	// 		Some(instance) => {
	// 			instance.config.vfs_config = vfs_config.clone();

	// 			// Update state
	// 			state.trigger_on_state_changed()?;
	// 			return Ok(());
	// 		}
	// 		None => {
	// 			return Err("No instance selected".to_string());
	// 		}
	// 	}
	// }
}

// ------------------------------
// NexusMods Interface
// ------------------------------

#[derive(Clone)]
struct ApiNexusModsStateImpl {
	state: MutexState,
}
#[taurpc::procedures(path = "nexusmods")]
trait ApiNexusMods {
	async fn validate_user() -> Result<(), String>;
}

#[taurpc::resolvers]
impl ApiNexusMods for ApiNexusModsStateImpl {
	async fn validate_user(self) -> Result<(), String> {
		let mut state = self.state.lock().await;

		let response = match state.application_config.nexusmods.validate_api_key().await {
			Ok(_) => Ok(()),
			Err(e) => Err(e),
		};

		state.save()?;

		return response;
	}
}

// ------------------------------
// Downloads Interface
// ------------------------------

#[derive(Clone)]
struct ApiDownloadsStateImpl {
	state: MutexState,
}
#[taurpc::procedures(path = "downloads", event_trigger = ApiDownloadsEventTrigger)]
trait ApiDownloads {
	async fn download_urls(url: Vec<String>) -> Result<(), String>;
	async fn resume_downloads() -> Result<(), String>;
	async fn delete_downloads(filenames: Vec<String>) -> Result<(), String>;
	async fn open_download_in_filemanager(filename: String) -> Result<(), String>;
	async fn open_extracted_folder(extracted_file: String) -> Result<(), String>;
	async fn install_file(app_handle: tauri::AppHandle, filename: String) -> Result<(), String>;
	async fn extract_file(filename: InstallerPayload) -> Result<UnpackedFileResponse, String>;
	async fn list_extracted_path_flattened(extracted_file: String) -> Result<Vec<String>, String>;
	async fn list_file_structure_relatively(
		extracted_file: String,
	) -> Result<Vec<FileStructureSegment>, String>;
	async fn read_extracted_file(extracted_file: String, paths: String) -> Result<Vec<u8>, String>;
	async fn install_mod_from_extracted(
		extracted_file: String,
		install_mod: InstallMod,
	) -> Result<(), String>;

	#[taurpc(event)]
	async fn on_downloads_update(downloads: Vec<Download>);
}

#[taurpc::resolvers]
impl ApiDownloads for ApiDownloadsStateImpl {
	async fn download_urls(self, urls: Vec<String>) -> Result<(), String> {
		let mut state = self.state.lock().await;

		let mut downloads_to_add: Vec<Download> = Vec::new();
		for url in urls {
			let mut parsed_download = Download {
				file_name: format!("temp-{}", rand::random::<u32>()),
				url: url.clone(),
				md5: None,
				// is_initialized: false,
				status: mods::downloader::DownloadStatus::Queued,
				size_total: String::from("0"),
				size_downloaded: String::from("0"),
				downloader: None,
				error: None,
				pending_update: false,
				added_at: mods::downloader::default_date(),
				completed_at: None,
				nexus_data: None,
			};

			// Handle nxm links
			if url.starts_with("nxm://") {
				// Parse nexusmods link
				let parsed_nexus_download =
					match state.application_config.nexusmods.parse_nxm_uri(url).await {
						Ok(v) => v,
						Err(e) => {
							println!("Error while parsing nexusmods link: {}", e);
							continue;
						}
					};

				if parsed_nexus_download.url == "" {
					println!("Error while parsing nexusmods link: Empty link");
					continue;
				}

				parsed_download.url = parsed_nexus_download.url.clone();
				parsed_download.file_name = parsed_nexus_download.filename.clone();
				parsed_download.md5 = parsed_nexus_download.md5.clone();
				parsed_download.nexus_data = Some(DownloadNexusData {
					mod_id: parsed_nexus_download.file_request.mod_id,
					file_id: parsed_nexus_download.file_request.file_id,
				});
			}

			// TODO: set name to the actual file name

			// TODO: Maybe add support for duplicated downloads by appending a number
			// Check if download already exists by filename
			if state
				.selected_instance
				.as_ref()
				.unwrap()
				.downloads
				.iter()
				.any(|d| d.file_name == parsed_download.file_name)
			{
				continue;
			}

			// Add download to state downloads
			downloads_to_add.push(parsed_download.clone());

			println!("Download added: {}", parsed_download.url);
		}

		let selected_instance = state.selected_instance.as_mut().unwrap();

		// Add downloads to instance
		selected_instance
			.downloads
			.append(downloads_to_add.borrow_mut());

		// Save instance state
		match selected_instance.save() {
			Ok(_) => {
				println!("Instance saved");
			}
			Err(e) => {
				println!("Error while saving state: {}", e);
			}
		}

		// drop mutex guard
		drop(state);

		let mut state = self.state.lock().await;
		state
			.selected_instance_or_fail()
			.start_downloads(self.state.clone())
			.await?;

		// return response;
		return Ok(());
	}

	async fn resume_downloads(self) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		selected_instance
			.start_downloads(self.state.clone())
			.await?;

		Ok(())
	}

	async fn delete_downloads(self, filenames: Vec<String>) -> Result<(), String> {
		let mut state = self.state.lock().await;

		let selected_instance = state.selected_instance_or_fail();

		for filename in filenames {
			// Filter downloads by filename
			let target_downloads = selected_instance
				.downloads
				.iter()
				.filter(|d| d.file_name == filename)
				.collect::<Vec<_>>();

			for download in target_downloads {
				match download.delete_file(selected_instance.get_downloads_absolute_path()) {
					Ok(_) => {
						println!("Download deleted: {}", download.url);
					}
					Err(e) => {
						println!("Error while deleting download: {}", e);
						return Err(e.to_string());
					}
				}
			}

			// Remove downloads from state
			selected_instance
				.downloads
				.retain(|d| d.file_name != filename);
		}

		// Save instance state
		match selected_instance.save() {
			Ok(_) => {
				println!("Instance saved");
			}
			Err(e) => {
				println!("Error while saving state: {}", e);
			}
		}

		return Ok(());
	}

	async fn open_download_in_filemanager(self, filename: String) -> Result<(), String> {
		let mut state = self.state.lock().await;

		let selected_instance = state.selected_instance_or_fail();
		let downloads_path = selected_instance.get_downloads_absolute_path();
		let download = selected_instance
			.downloads
			.iter()
			.find(|d| d.file_name == filename)
			.expect("Download with the given filename was not found");
		let file_path = downloads_path.join(download.file_name.clone());

		return crate::controllers::file_controller::open_in_filemanager(file_path);
	}

	async fn open_extracted_folder(self, unpacked_filename: String) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let downloads_path = selected_instance.get_downloads_absolute_path();

		// Extracted path
		let target_filepath: PathBuf = downloads_path
			.join(PathBuf::from("extracted"))
			.join(PathBuf::from(unpacked_filename));

		return crate::controllers::file_controller::open_folder(target_filepath);
	}

	async fn install_file(
		self,
		app_handle: tauri::AppHandle,
		filename: String,
	) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let download_path = PathBuf::from(filename.clone());

		let payload = match download_path.is_relative() {
			true => {
				let download = selected_instance
					.downloads
					.iter()
					.find(|d| d.file_name == filename)
					.expect("Download with the given filename was not found")
					.clone();

				InstallerPayload {
					file_name: download.file_name.clone(),
					absolute_path: selected_instance
						.get_downloads_absolute_path()
						.join(download.file_name)
						.to_str()
						.unwrap()
						.to_string(),
					is_relative: true,
				}
			}
			false => InstallerPayload {
				file_name: download_path.to_str().unwrap().to_string(),
				absolute_path: download_path.to_str().unwrap().to_string(),
				is_relative: false,
			},
		};

		// window.app_handle().
		std::thread::spawn(move || {
			let installer_window = tauri::WindowBuilder::new(
				&app_handle,
				"local",
				tauri::WindowUrl::App(
					format!("src/installer/index.html?filename={}", &filename).into(),
				),
			)
			.build()
			.unwrap();

			let installer_window_clone = installer_window.clone();

			installer_window.once("ready", move |_| {
				println!("Installer window is ready: {}", &payload.file_name);

				let data_to_send = serde_json::to_string(&payload)
					.expect("Error while serializing installer data");
				installer_window_clone
					.emit("installer-data", data_to_send)
					.expect("Error while emitting installer data");
			});
		});

		return Ok(());
	}

	async fn extract_file(self, payload: InstallerPayload) -> Result<UnpackedFileResponse, String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let downloads_absolute_path = selected_instance.get_downloads_absolute_path();
		let download_absolute_path = match payload.is_relative {
			true => downloads_absolute_path.join(payload.file_name.clone()),
			false => PathBuf::from(payload.file_name.clone()),
		};

		let mut unpacked_filename = match payload.is_relative {
			true => payload.file_name,
			false => PathBuf::from(payload.file_name)
				.file_name()
				.unwrap()
				.to_str()
				.unwrap()
				.to_string(),
		};

		// Append _unpacked to the filename
		unpacked_filename.push_str("_unpacked");

		// Extracted path
		let extracted_path_absolute: PathBuf = downloads_absolute_path
			.join(PathBuf::from("extracted"))
			.join(PathBuf::from(unpacked_filename.clone()));

		// Delete pre-extracted files, if any
		if extracted_path_absolute.clone().exists() {
			println!(
				"Deleting pre-existing extracted folder: {:?}",
				extracted_path_absolute
			);
			file_controller::delete_folder_safe(
				extracted_path_absolute.clone(),
				downloads_absolute_path,
			)
			.map_err(|e| {
				format!(
					"Failed to delete existing extracted folder: {}",
					e.to_string()
				)
			})?;
		}

		println!("Extracting download archive: {:?}", download_absolute_path);

		// Extract the archive
		crate::controllers::file_controller::extract_archive(
			download_absolute_path,
			extracted_path_absolute.clone(),
		)?;

		// Check if we have a sub-root
		let entries =
			match file_controller::list_entries_absolute_path(extracted_path_absolute.clone()) {
				Ok(entries) => entries,
				Err(e) => {
					return Err(format!("Failed to list entries: {}", e.to_string()));
				}
			};

		// If we have a sub-root
		if entries.len() == 1 {
			// We have a sub-root
			// Move the sub-root to the root
			let sub_root = entries[0].clone();
			if sub_root.is_dir() {
				println!("Moving sub-root to root: {:?}", sub_root);

				// Move sub-root to main root
				file_controller::move_folder(sub_root.clone(), extracted_path_absolute.clone())?;

				// Delete old sub-root
				// file_controller::delete_folder_safe(sub_root, extracted_path.clone())
				// 	.map_err(|e| format!("Failed to delete sub-root: {}", e.to_string()))?;
			}
		};

		// Case-fold entire extracted path
		match crate::controllers::file_controller::case_fold_folder_recursive(
			extracted_path_absolute.clone(),
		) {
			Ok(_) => {}
			Err(e) => {
				return Err(format!("Failed to case-fold folder: {}", e.to_string()));
			}
		}

		// Return extracted path
		return Ok(UnpackedFileResponse {
			relative_folder: unpacked_filename,
			absolute_folder: extracted_path_absolute.to_str().unwrap().to_string(),
		});
	}

	async fn list_extracted_path_flattened(
		self,
		unpacked_filename: String,
	) -> Result<Vec<String>, String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let downloads_path = selected_instance.get_downloads_absolute_path();

		// Extracted path
		let target_filepath: PathBuf = downloads_path
			.join(PathBuf::from("extracted"))
			.join(PathBuf::from(unpacked_filename));

		return match file_controller::list_files_recursively_relative_flattened(target_filepath) {
			Ok(files) => Ok(files),
			Err(e) => Err(e.to_string()),
		};
	}

	async fn list_file_structure_relatively(
		self,
		unpacked_filename: String,
	) -> Result<Vec<FileStructureSegment>, String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let downloads_path = selected_instance.get_downloads_absolute_path();

		// Extracted path
		let target_filepath: PathBuf = downloads_path
			.join(PathBuf::from("extracted"))
			.join(PathBuf::from(unpacked_filename));

		return match file_controller::list_file_structure_relatively(target_filepath) {
			Ok(files) => Ok(files),
			Err(e) => Err(e.to_string()),
		};
	}

	async fn read_extracted_file(
		self,
		unpacked_filename: String,
		relative_path: String,
	) -> Result<Vec<u8>, String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let downloads_path = selected_instance.get_downloads_absolute_path();

		// Extracted path
		let mut target_filepath: PathBuf = downloads_path
			.join(PathBuf::from("extracted"))
			.join(PathBuf::from(unpacked_filename));

		// Push paths
		for p in relative_path.split('/') {
			target_filepath.push(p.to_lowercase());
		}

		return match crate::controllers::file_controller::read_file_bytes(target_filepath) {
			Ok(bytes) => Ok(bytes),
			Err(e) => Err(e.to_string()),
		};
	}

	async fn install_mod_from_extracted(
		self,
		unpacked_filename: String,
		install_mod: InstallMod,
	) -> Result<(), String> {
		let mut state = self.state.lock().await;
		let selected_instance = state.selected_instance_or_fail();

		let downloads_absolute_path = selected_instance.get_downloads_absolute_path();

		// Extracted path
		let extracted_path: PathBuf = downloads_absolute_path
			.join(PathBuf::from("extracted"))
			.join(PathBuf::from(unpacked_filename.clone()));

		// Create mod
		let mod_instance = selected_instance.create_mod_version(
			install_mod.name,
			install_mod.version,
			install_mod.info,
		)?;

		// Get version absolute path
		let version_absolute_path = mod_instance.get_selected_version_absolute_path();

		// Get deployment file structure, so we can check folder/file casing
		let deployment_file_structure = selected_instance.get_mods_deployment_file_structure()?;

		// Move files
		for file in install_mod.files {
			let file_source = file.source.replace("\\", "/").to_lowercase();
			let mut case_folded_file_destination =
				file.destination.replace("\\", "/").to_lowercase();

			// Check if the file is in the deployment file structure
			let mut found_case_folded_path: Option<String> = None;
			for file_structure in deployment_file_structure.clone() {
				let split_path_str = case_folded_file_destination
					.split("/")
					.collect::<Vec<&str>>();

				// Convert split_path from Vec<&str> to Vec<String>
				let split_path = split_path_str
					.iter()
					.map(|s| s.to_string())
					.collect::<Vec<String>>();

				match file_structure.case_fold_path(split_path) {
					Some(case_folded_path) => {
						found_case_folded_path = Some(case_folded_path);
						break;
					}
					None => continue,
				}
			}

			// If a path was found matching deployment casing, use it instead
			if found_case_folded_path.is_some() {
				case_folded_file_destination = found_case_folded_path.unwrap();
			}

			let source_file_absolute_path =
				file_controller::join_paths(extracted_path.clone(), PathBuf::from(file_source));
			let destination_file_absolute_path = file_controller::join_paths(
				version_absolute_path.clone(),
				PathBuf::from(case_folded_file_destination),
			);

			// Move file/folder
			// Check if we are moving a file
			if source_file_absolute_path.is_file() {
				file_controller::move_file(
					source_file_absolute_path,
					destination_file_absolute_path,
				)
				.map_err(|e| format!("Failed to move file: {}", e.to_string()))?;
			} else {
				file_controller::copy_recursive(
					source_file_absolute_path,
					destination_file_absolute_path,
					true,
				)
				.map_err(|e| format!("Failed to copy with hardlinks folder: {}", e.to_string()))?;
			}
		}

		// Delete extracted files
		file_controller::delete_folder_safe(extracted_path.clone(), downloads_absolute_path)
			.map_err(|e| format!("Failed to delete extracted files: {}", e.to_string()))?;

		// Load mods
		selected_instance.load_mods()?;

		// Trigger on state changed
		state.trigger_on_state_changed()?;

		return Ok(());
	}
}

#[tokio::main]
async fn main() {
	let stateMutex = Arc::new(Mutex::new(
		state::ApplicationState::load_or_new().expect("Error while loading application state"),
	));

	let (apphandle_tx, apphandle_rx) = oneshot::channel::<AppHandle>();

	let router = Router::new()
		.merge(
			ApiImpl {
				state: stateMutex.clone(),
			}
			.into_handler(),
		)
		.merge(
			ApiInstancesStateImpl {
				state: stateMutex.clone(),
			}
			.into_handler(),
		)
		.merge(
			ApiNexusModsStateImpl {
				state: stateMutex.clone(),
			}
			.into_handler(),
		)
		.merge(
			ApiDownloadsStateImpl {
				state: stateMutex.clone(),
			}
			.into_handler(),
		);

	// Spawn async task to set events trigger
	tokio::spawn(async move {
		let app_handle = apphandle_rx.await.unwrap();
		let events_trigger = ApiEventTrigger::new(app_handle.clone());
		let downloads_events_trigger: ApiDownloadsEventTrigger =
			ApiDownloadsEventTrigger::new(app_handle.clone());

		// Set event trigger
		let mut state = stateMutex.lock().await;
		state.set_events_triggers(events_trigger, downloads_events_trigger);

		drop(state);

		// Loop every second
		loop {
			let mut state = stateMutex.lock().await;

			// -------------------------------
			// Check for running executables
			// -------------------------------
			state.check_running_executables();

			if state.selected_instance.is_some() {
				let selected_instance = state.selected_instance.as_mut().unwrap();

				// ---------------------------------------------
				// Check if there are downloads to update
				// ---------------------------------------------
				let flagged_downloads = selected_instance
					.downloads
					.iter_mut()
					.filter(|d| d.pending_update);

				// Build array of downloads to send
				let mut download_vec: Vec<Download> = Vec::new();
				for d in flagged_downloads {
					d.pending_update = false;
					download_vec.push(d.clone());
				}

				if download_vec.len() > 0 {
					// Save state
					match selected_instance.save() {
						Ok(_) => {}
						Err(e) => {
							println!("Error while saving instance: {}", e);
						}
					}

					// Send downloads update event
					let _ = state
						.download_event_trigger
						.as_ref()
						.unwrap()
						.on_downloads_update(download_vec);
				}
			}

			drop(state);

			// Sleep for a second
			tokio::time::sleep(Duration::from_secs(1)).await;
		}
	});

	// Create system tray icon
	// let tray = SystemTray::new();

	// let ipc_manager = mods::ipc::

	tauri::Builder::default()
		.plugin(tauri_plugin_window_state::Builder::default().build())
		.invoke_handler(router.into_handler())
		.setup(|_app| {
			// Payload to send or process if no existing process
			let mut ipc_payload: Option<IPCPayload> = None;

			// Here we handle CLI arguments
			match _app.get_cli_matches() {
				Ok(matches) => {
					match matches.subcommand {
						// Handle subcommand
						Some(subcommand) => {
							let subcommand_response = handle_cli_subcommands(subcommand);

							// Check if we should force exit
							if subcommand_response.force_exit {
								std::process::exit(0);
							}

							// Set IPC payload
							ipc_payload = subcommand_response.ipc_payload;
						}
						None => {}
					};
				}
				Err(_) => {}
			}

			// Check if there is an existing process
			let mut ipc_client = IPCClient::new();

			// Check if there is an existing socket
			if ipc_client.socket_path_exists() {
				// There is an existing process
				match ipc_client.ping_socket() {
					Ok(()) => {
						// There is an existing process, so we need to send the payload
						if ipc_payload.is_some() {
							match ipc_client.send_payload_to_stream(ipc_payload.unwrap()) {
								Ok(_) => {
									println!("Sent IPC payload to existing process");
								}
								Err(e) => {
									panic!("Failed to send IPC payload to existing process: {}", e);
								}
							}
						}

						// Exit
						println!("There is an existing process, exiting...");
						std::process::exit(0);
					}
					Err(e) => {
						println!("Failed to ping existing socket: \"{}\". ", e);
						// panic!("Failed to ping socket: {}", e);
					}
				}
			}

			apphandle_tx.send(_app.handle()).unwrap();

			// Initialize IPC Server
			let ipc_server = IPCServer {
				app_handle: _app.handle(),
			};
			thread::spawn(move || {
				ipc_server.initialize_listener(ipc_payload);
			});

			#[cfg(debug_assertions)]
			{
				let main_window = _app.get_window("main").unwrap();
				main_window.open_devtools();
			}

			Ok(())
		})
		.register_uri_scheme_protocol("reqimg", move |app, request| {
			let url: Url = request.uri().parse().unwrap();

			let mut base_folder: Option<String> = None;
			let mut file_name: Option<String> = None;

			// Loop query pairs
			for (key, value) in url.query_pairs() {
				let lowercaseKey = key.to_lowercase();
				if lowercaseKey == "basefolder" {
					base_folder = Some(value.to_string());
				} else if lowercaseKey == "filename" {
					file_name = Some(value.to_string());
				}
			}

			if base_folder.is_none() || file_name.is_none() {
				return ResponseBuilder::new().status(404).body(Vec::new());
			}

			// Extracted path
			let mut target_filepath: PathBuf = PathBuf::from(base_folder.unwrap());

			// Push paths
			for p in file_name.unwrap().split('/') {
				if p.is_empty() {
					continue;
				}

				target_filepath.push(p.to_lowercase());
			}

			let file_kind = match infer::get_from_path(&target_filepath) {
				Ok(kind) => kind,
				Err(_) => {
					return ResponseBuilder::new().status(404).body(Vec::new());
				}
			};

			if file_kind.is_none() {
				println!("Failed to infer file type");
				return ResponseBuilder::new().status(500).body(Vec::new());
			}

			let mime_type = file_kind.unwrap().mime_type();

			if !mime_type.starts_with("image/") {
				return ResponseBuilder::new().status(401).body(Vec::new());
			}

			let file_data =
				match crate::controllers::file_controller::read_file_bytes(target_filepath) {
					Ok(bytes) => bytes,
					Err(e) => {
						println!("Failed to read file: {}", e);
						return ResponseBuilder::new().status(500).body(Vec::new());
					}
				};

			return tauri::http::ResponseBuilder::new()
				.mimetype(mime_type)
				.body(file_data);
		})
		.plugin(tauri_plugin_window_state::Builder::default().build())
		.run(tauri::generate_context!())
		.expect("error while running application");
}

struct SubcommandResponse {
	force_exit: bool,
	ipc_payload: Option<IPCPayload>,
}

fn handle_cli_subcommands(subcommand: Box<SubcommandMatches>) -> SubcommandResponse {
	let mut response = SubcommandResponse {
		force_exit: false,
		ipc_payload: None,
	};

	match subcommand.name.as_str() {
		"nxm" => {
			println!("Found nxm subcommand");
			let nxm_link_match = subcommand.matches.args.get("link");

			match nxm_link_match {
				Some(link) => {
					response.ipc_payload = Some(IPCPayload {
						command: "nxm".to_string(),
						args: vec![link.value.to_string()],
					})
				}
				None => {
					println!("No link found for nxm subcommand");
				}
			}
		}
		// "ping" => {
		// 	// return Some(format!("ping"));
		// 	response.force_exit = true;
		// }
		"config" => {
			let config_location = state::root_config_path();
			println!("Application config is found at: {:?}", config_location);

			response.force_exit = true;
		}
		_ => {
			panic!("Unknown subcommand: {}", subcommand.name);
		}
	}

	return response;
}
