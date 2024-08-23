use dirs;
// use erased_serde::serialize_trait_object;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	process::{Child, Command},
	sync::{Arc, Mutex},
};

use crate::{
	controllers,
	deployer::vfs::base_vfs::BaseVFS,
	instances::{self, GameInstance, InstanceExecutable},
	mods::downloader,
	ApiDownloadsEventTrigger, ApiEventTrigger,
};

use self::config::ApplicationConfig;

const CURRENT_CONFIG_VERSION: &str = "0.0.1";
const ROOT_CONFIG_FOLDER_NAME: &str = "rmm.neilseligmann.com";

pub mod config;

pub fn root_config_path() -> PathBuf {
	let mut path = dirs::config_dir().unwrap();
	path.push(ROOT_CONFIG_FOLDER_NAME);
	return path;
}

pub fn default_instances_path() -> PathBuf {
	let mut path = root_config_path();
	path.push("instances");
	return path;
}

const fn _default_none<T>() -> Option<T> {
	None
}

fn _default_config() -> ApplicationConfig {
	config::ApplicationConfig::new()
}

fn _default_front_config() -> FrontendConfig {
	FrontendConfig::new()
}

fn _default_vector_empty<T>() -> Vec<T> {
	Vec::new()
}

#[taurpc::ipc_type]
pub struct AvailableInstancesResponse {
	pub instances: Vec<GameInstance>,
	pub errors: Vec<InstanceError>,
}

#[taurpc::ipc_type]
pub struct InstanceError {
	pub error: String,
	pub instance_path: PathBuf,
}

pub struct RunningExecutable {
	pub executable_name: String,
	pub child_process: Child,
}

// pub type RunningExecutablesMutex = Arc<Mutex<Vec<RunningExecutable>>>;

#[taurpc::ipc_type]
pub struct ApplicationState {
	#[serde(default = "_default_config")]
	pub application_config: config::ApplicationConfig,

	#[serde(default = "_default_front_config")]
	pub frontend_config: FrontendConfig,

	#[serde(default = "_default_none")]
	pub selected_instance_path: Option<PathBuf>,

	// We don't want to save this field, we only need it at Runtime
	#[serde(default = "_default_none")]
	pub selected_instance: Option<GameInstance>,

	#[serde(skip)]
	pub event_trigger: Option<ApiEventTrigger>,

	#[serde(skip)]
	pub download_event_trigger: Option<ApiDownloadsEventTrigger>,

	#[serde(default)]
	pub instances_errors: Vec<InstanceError>,

	// Specta does not support traits yet.
	// so we have to serialize/deserialize manually
	// #[serde(skip)]
	// pub mounted_vfs: Vec<Box<dyn BaseVFS>>,
	#[serde(default)]
	pub is_vfs_mounted: bool,

	#[serde(skip)]
	pub running_executables: Arc<Mutex<Vec<RunningExecutable>>>,

	#[serde(default)]
	pub running_executables_id: HashMap<String, Vec<u32>>,
}

impl Default for ApplicationState {
	fn default() -> Self {
		Self::new()
	}
}

impl ApplicationState {
	pub fn new() -> Self {
		return Self {
			application_config: config::ApplicationConfig::new(),
			selected_instance_path: None,
			selected_instance: None,
			event_trigger: None,
			download_event_trigger: None,
			frontend_config: FrontendConfig::new(),
			instances_errors: Vec::new(),
			// mounted_vfs: Vec::new(),
			is_vfs_mounted: false,
			running_executables: Arc::new(Mutex::new(Vec::new())),
			running_executables_id: HashMap::new(),
		};
	}

	pub fn set_events_triggers(
		&mut self,
		event_trigger: ApiEventTrigger,
		download_event_trigger: ApiDownloadsEventTrigger,
	) {
		self.event_trigger = Some(event_trigger);
		self.download_event_trigger = Some(download_event_trigger);
	}

	pub fn load_or_new() -> Result<Self, String> {
		let state_path = root_config_path().join("state.json");

		if !state_path.exists() {
			return Ok(Self::new());
		}

		let json = controllers::file_controller::read_file(state_path)
			.map_err(|e| format!("Failed to read json state file: {}", e.to_string()))?;
		let mut state: Self = serde_json::from_str(&json)
			.map_err(|e| format!("Failed to parse state json: {}", e.to_string()))?;

		// Update mounted vfs state
		state.fetch_mounted_vfs();

		// Clear running executables
		state.running_executables_id = HashMap::new();
		state.instances_errors = Vec::new();

		// Reload selected instance, if possible
		if state.selected_instance_path.is_some() {
			let instance_path = state.selected_instance_path.clone().unwrap();
			match GameInstance::load_from_path(instance_path.clone()) {
				Ok(instance) => {
					state.selected_instance = Some(instance);
				}
				Err(e) => {
					println!(
						"Failed to load at startup the selected instance from path \"{}\": {}",
						instance_path.to_str().unwrap(),
						e
					);
					state.instances_errors.push(InstanceError {
						error: e,
						instance_path: instance_path,
					});
				}
			}
		}

		Ok(state)
	}

	pub fn save(&mut self) -> Result<(), String> {
		let state_path = root_config_path().join("state.json");

		// Clone it so we can remove the selected_instance field
		let mut cloned = self.clone();

		// Clear runtime fields
		cloned.selected_instance = None;
		cloned.running_executables_id = HashMap::new();
		cloned.is_vfs_mounted = false;

		let json = serde_json::to_string(&cloned).map_err(|e| e.to_string())?;

		// Save json file
		controllers::file_controller::save_file_with_backup(state_path, json.as_bytes())
			.map_err(|e| format!("Failed to save state file: {}", e))?;

		// let vfs_state_path = root_config_path().join("vfs_state.json");
		// let vfs_json = serde_json::to_string(&self.mounted_vfs).map_err(|e| e.to_string())?;
		// controllers::file_controller::save_file(vfs_state_path, vfs_json.as_bytes())?;

		// Trigger on state changed event
		self.trigger_on_state_changed()?;

		return Ok(());
	}

	pub fn trigger_on_state_changed(&self) -> Result<(), String> {
		match self.event_trigger {
			Some(ref trigger) => {
				let state = self.clone();
				let _ = trigger.on_state_changed(state);
			}
			None => {}
		}

		Ok(())
	}

	pub fn get_game_instance(&mut self) -> Result<Option<GameInstance>, String> {
		if let Some(instance_path) = &self.selected_instance_path {
			let instance = GameInstance::load_from_path(instance_path.to_path_buf())?;
			return Ok(Some(instance));
		}

		Ok(None)
	}

	pub fn select_instance(&mut self, instance: GameInstance) -> Result<(), String> {
		self.selected_instance = Some(instance.clone());
		self.selected_instance_path = Some(instance.clone().config.paths.root);

		return self.save();
	}

	pub fn add_instance_path(&mut self, instance_path: PathBuf) -> Result<(), String> {
		self.application_config
			.available_instances_paths
			.push(instance_path);

		return self.save();
	}

	pub fn selected_instance_or_fail(&mut self) -> &mut GameInstance {
		return self
			.selected_instance
			.as_mut()
			.expect("No instance selected");
	}

	// ----------------
	// VFS
	// ----------------

	pub fn fetch_mounted_vfs(&mut self) -> Vec<Box<dyn BaseVFS>> {
		// Load mounted VFS instances
		let vfs_state_path = root_config_path().join("vfs_state.json");
		if !vfs_state_path.exists() {
			return Vec::new();
		}

		let vfs_json = match controllers::file_controller::read_file(vfs_state_path)
			.map_err(|e| e.to_string())
		{
			Ok(json) => json,
			Err(e) => {
				println!("Failed to load VFS state: {}", e);
				return Vec::new();
			}
		};

		let mounted_vfs: Vec<Box<dyn BaseVFS>> =
			match serde_json::from_str(&vfs_json).map_err(|e| e.to_string()) {
				Ok(vfs) => vfs,
				Err(e) => {
					println!("Failed to load VFS state: {}", e);
					return Vec::new();
				}
			};

		// Update mounted vfs state
		// state.mounted_vfs = mounted_vfs;
		let are_vfs_mounted = mounted_vfs.len() > 0;
		if are_vfs_mounted != self.is_vfs_mounted {
			self.is_vfs_mounted = are_vfs_mounted;
			self.save()
				.expect("Failed to save instance while saving VFS mounted state!");
		}

		return mounted_vfs;
	}

	pub fn save_mounted_vfs(&mut self, vfs: Vec<Box<dyn BaseVFS>>) -> Result<(), String> {
		let vfs_state_path = root_config_path().join("vfs_state.json");
		let vfs_json = serde_json::to_string(&vfs).map_err(|e| e.to_string())?;
		controllers::file_controller::save_file(vfs_state_path, vfs_json.as_bytes())
			.map_err(|e| e.to_string())?;

		self.is_vfs_mounted = vfs.len() > 0;
		self.save()?;

		return Ok(());
	}

	pub fn check_vfs_mounted(&mut self) -> bool {
		self.fetch_mounted_vfs();

		return self.is_vfs_mounted;
	}

	pub fn mount_vfs(&mut self) -> Result<(), String> {
		let existing_vfs = self.fetch_mounted_vfs();

		if existing_vfs.len() > 0 {
			return Err("VFS already mounted".to_string());
		}

		let fallback_vfs: config::vfs_config::VFSConfig =
			self.application_config.default_vfs_config.clone();
		let selected_instance = &self.selected_instance_or_fail();

		// Mount VFS
		let mounted_vfs = selected_instance.mount_vfs(fallback_vfs)?;

		// Add newly mounted VFSes to existing list
		// existing_vfs.append(&mut mounted_vfs);
		// existing_vfs = mounted_vfs;

		// Save and update state
		self.save_mounted_vfs(mounted_vfs)?;

		return Ok(());
	}

	pub fn unmount_vfs(&mut self) -> Result<(), String> {
		let existing_vfs = self.fetch_mounted_vfs();

		for mounted_vfs in existing_vfs.iter() {
			mounted_vfs.unmount()?;
		}

		self.save_mounted_vfs(Vec::new())?;

		return Ok(());
	}

	// ----------------
	// Executables
	// ----------------

	pub fn running_executables_add(&mut self, executable_name: String, child: Child) -> () {
		self.running_executables_id
			.entry(executable_name.clone())
			.or_insert(Vec::new())
			.push(child.id());

		self.save();

		// Push to running executables
		let mut running_executables = self.running_executables.lock().unwrap();
		running_executables.push(RunningExecutable {
			executable_name,
			child_process: child,
		});
	}

	pub fn running_executable_remove(
		&mut self,
		executable_name: String,
		pid: u32,
		_save: bool,
	) -> () {
		self.running_executables_id
			.entry(executable_name)
			.and_modify(|v| {
				v.retain(|&x| x != pid);
			});

		if _save {
			self.save().expect("Failed to save state");
		}

		// Remove from running executables
		let mut running_executables = self.running_executables.lock().unwrap();
		running_executables.retain(|x| x.child_process.id() != pid);
	}

	pub fn stop_running_executable(&mut self, executable_name: String) -> () {
		let binding = self.running_executables.clone();
		let mut running_executables = binding.lock().unwrap();

		for running_executable in running_executables.iter_mut() {
			if running_executable.executable_name == executable_name {
				running_executable
					.child_process
					.kill()
					.expect("Failed to kill process");
				running_executable
					.child_process
					.wait()
					.expect("Failed to wait for process");
			}
		}
	}

	pub fn check_running_executables(&mut self) -> () {
		let binding = self.running_executables.clone();
		let mut running_executables = binding.lock().unwrap();

		struct ExitedProcess {
			executable_name: String,
			pid: u32,
		}

		let mut exited_children: Vec<ExitedProcess> = Vec::new();

		// Loop over running executables
		for running_executable in running_executables.iter_mut() {
			match running_executable.child_process.try_wait() {
				Ok(Some(_)) => {
					// child has exited, remove from running executables
					exited_children.push(ExitedProcess {
						executable_name: running_executable.executable_name.clone(),
						pid: running_executable.child_process.id(),
					});
				}
				Ok(None) => {
					// child hasn't exited yet
				}
				Err(e) => println!("Error attempting to wait for child process: {e}"),
			}
		}

		drop(running_executables);

		for exited_child in exited_children.iter() {
			self.running_executable_remove(
				exited_child.executable_name.clone(),
				exited_child.pid,
				false,
			);
		}

		// TODO: Update running executables using a different event
		if exited_children.len() > 0 {
			self.trigger_on_state_changed()
				.expect("Failed to trigger on state changed");
		}

		// Save state
		// self.save().expect("Failed to save state");
	}

	// ----------------
	// Instances
	// ----------------

	pub async fn list_available_instances(&mut self) -> Result<AvailableInstancesResponse, String> {
		println!("List available instances");
		// let state = self.state.lock().await;

		let mut instances = Vec::new();
		let mut errors: Vec<InstanceError> = Vec::new();

		for instance_path in self.application_config.available_instances_paths.clone() {
			match GameInstance::load_from_path(instance_path.to_path_buf()) {
				Ok(instance) => instances.push(instance),
				Err(e) => {
					errors.push(InstanceError {
						instance_path,
						error: e.to_string(),
					});
				}
			}
		}

		// Loop instances inside default config path
		let default_instances_path = default_instances_path();
		println!("default_config_path: {}", default_instances_path.display());
		if default_instances_path.exists() {
			println!(
				"Found default config path: {}",
				default_instances_path.clone().display()
			);
			for entry in
				std::fs::read_dir(default_instances_path.clone()).expect("Failed to read directory")
			{
				let entry = entry.map_err(|e| e.to_string())?;
				println!("Found entry: {:?}", entry);
				let instance_path = entry.path();

				println!("Found instance: {}", instance_path.clone().display());

				match GameInstance::load_from_path(instance_path.clone()) {
					Ok(instance) => {
						// Check if already exists
						if instances.contains(&instance) {
							continue;
						}

						instances.push(instance);
					}
					Err(e) => {
						errors.push(InstanceError {
							instance_path,
							error: e.to_string(),
						});
					}
				}
			}
		}

		// Add errors to state and trigger on state changed
		self.instances_errors = errors.clone();
		self.trigger_on_state_changed();

		return Ok(AvailableInstancesResponse { instances, errors });
	}
}

#[taurpc::ipc_type]
pub struct FrontendConfig {
	pub sidebar_pinned: bool,
}

impl FrontendConfig {
	pub fn new() -> Self {
		return Self {
			sidebar_pinned: false,
		};
	}
}
