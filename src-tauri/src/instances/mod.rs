use crate::controllers::file_controller::{self, FileStructureSegment};
use crate::controllers::plugin_controller::{self, BethesdaPlugin};
use crate::deployer::vfs;
use crate::deployer::vfs::base_vfs::{BaseVFS, VFSMountConfig, VFSMountPaths};
use crate::deployer::vfs::union_fs_fuse::UnionFSFuse;
use crate::mods::downloader;
use crate::state::config::vfs_config::{VFSConfig, VFSImplementation};
use crate::state::ApplicationState;
use base64::engine::general_purpose;
use base64::Engine;
use image::{DynamicImage, ImageBuffer};
use instance_mod::ModInfo;
use loadorder::GameSettings;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::Path;
use std::process::{Child, Command};
use std::str::FromStr;
use std::sync::Arc;
use std::{ffi::OsString, path::PathBuf};
use tokio::sync::Mutex;
// use crate::deployer::vfs

use self::instance_mod::InstanceMod;

pub mod instance_mod;

pub fn default_true() -> bool {
	true
}

fn _default_none<T>() -> Option<T> {
	None
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq, Default, Copy)]
pub enum GameIdentifier {
	#[default]
	Generic,
	Oblivion,
	Morrowind,
	Skyrim,
	SkyrimSE,
	Fallout3,
	FalloutNV,
	Fallout4,
}

impl GameIdentifier {
	fn default() -> Self {
		GameIdentifier::Generic
	}
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct InstanceExecutable {
	// Can be absolute or use variables
	pub path: Option<PathBuf>,
	pub command: Option<String>,
	pub args: Option<String>,
	// Visual elements
	pub icon: Option<String>,
	pub name: String,
	pub show_shortcut: Option<bool>,
	#[serde(default = "default_true")]
	pub use_compability: bool,
	#[serde(default = "default_true")]
	pub use_proton_tricks: bool,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct GameInstanceConfig {
	pub name: String,
	#[serde(default)]
	pub steam_id: Option<String>,
	// #[serde(default)]
	pub paths: GameInstancePaths,
	#[serde(default)]
	pub vfs_config: Option<VFSConfig>,
	#[serde(default)]
	pub executables: Vec<InstanceExecutable>,
	#[serde(default)]
	pub game_identifier: GameIdentifier,
	#[serde(default)]
	pub folding_config: CaseFoldingConfig,
	#[serde(default)]
	pub downloads_config: DownloadsConfig,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct CaseFoldingConfig {
	#[serde(default = "default_true")]
	pub enabled: bool,
}

impl Default for CaseFoldingConfig {
	fn default() -> Self {
		CaseFoldingConfig { enabled: true }
	}
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct DownloadsConfig {
	#[serde(default)]
	#[specta(type = String)]
	pub concurrent_downloads: usize,
	#[serde(default)]
	#[specta(type = String)]
	pub threads_per_download: usize,
}

impl Default for DownloadsConfig {
	fn default() -> Self {
		DownloadsConfig {
			concurrent_downloads: 2,
			threads_per_download: 4,
		}
	}
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct GameInstanceInternalPaths {
	pub mods: PathBuf,
	pub downloads: PathBuf,
	pub settings: PathBuf,
	pub saves: PathBuf,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct GameInstanceDeploymentPaths {
	pub mods: PathBuf,
	pub settings: Option<PathBuf>,
	pub saves: Option<PathBuf>,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct GameInstancePaths {
	pub root: PathBuf,
	pub game: PathBuf,
	pub internal: GameInstanceInternalPaths,
	pub deployment: GameInstanceDeploymentPaths,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct GameInstance {
	// #[serde(default)]
	pub config: GameInstanceConfig,
	#[serde(default)]
	pub mods: Vec<InstanceMod>,
	#[serde(default)]
	pub mods_indexes: HashMap<String, u32>,
	#[serde(default)]
	pub mods_errors: HashMap<String, String>,

	#[serde(default)]
	pub downloads: Vec<downloader::Download>,
	// Plugins
	// #[serde(default)]
	// pub plugins: HashMap<String, Vec<BethesdaPlugin>>,
	// #[serde(default)]
	// pub game_settings: Option<GameSettings>,
}

impl PartialEq for GameInstance {
	fn eq(&self, other: &Self) -> bool {
		self.config.paths.root == other.config.paths.root
	}
}

impl GameInstance {
	pub fn load_from_path(instance_path: PathBuf) -> Result<Self, String> {
		if !instance_path.exists() {
			return Err(format!(
				"Instance path \"{}\" does not exist",
				instance_path.to_str().unwrap()
			));
		}

		let json_path = instance_path.join("instance.json");

		// Validate instance has a valid instance.json file
		if !json_path.exists() {
			return Err(format!(
				"Instance path \"{}\" does not contain an instance.json file",
				instance_path.to_str().unwrap()
			));
		}

		// Read instance.json file
		// let json =
		// 	crate::controllers::file_controller::read_file(instance_path.join("instance.json"))
		// 		.map_err(|e| e.to_string())?;
		// let instance_io: GameInstance = serde_json::from_str(&json).map_err(|e| e.to_string())?;
		// let instance_io_value = match serde_json::to_value(&instance_io) {
		// 	Ok(value) => value,
		// 	Err(e) => return Err(e.to_string()),
		// };

		let json =
			crate::controllers::file_controller::read_file(instance_path.join("instance.json"))
				.map_err(|e| format!("Failed to parse instance from json: {}", e.to_string()))?;

		// Deserialize the game instance from the IO struct
		let mut instance: GameInstance = match serde_json::from_str(&json) {
			Ok(instance) => instance,
			Err(e) => return Err(e.to_string()),
		};

		// Set instance root path
		instance.config.paths.root = instance_path.clone();

		// Override instance root path, if needed
		if instance.config.paths.root != instance_path {
			instance.config.paths.root = instance_path.clone();
			instance.save()?;
		}

		instance.load_mods()?;

		// Load plugins if not generic
		// if instance.config.game_identifier != GameIdentifier::Generic {
		// 	instance.get_plugins()?;
		// }

		Ok(instance)
	}

	pub fn exists(instance_path: PathBuf) -> bool {
		return instance_path.exists();
	}

	pub fn new(name: String, paths: GameInstancePaths) -> Result<Self, String> {
		// let paths = GameInstancePaths {
		// 	root: paths.root.clone(),
		// 	deployment: GameInstanceDeploymentPaths {
		// 		game: paths.game,
		// 		mods: paths.mods.unwrap_or_else(|| paths.root.join("mods")),
		// 		downloads: paths
		// 			.downloads
		// 			.unwrap_or_else(|| paths.root.join("downloads")),
		// 		settings: paths.settings,
		// 		saves: paths.saves,
		// 	},
		// };

		let mut new_instance = Self {
			config: GameInstanceConfig {
				name,
				steam_id: None,
				paths,
				executables: Vec::new(),
				vfs_config: None,
				game_identifier: GameIdentifier::default(),
				folding_config: CaseFoldingConfig::default(),
				downloads_config: DownloadsConfig::default(),
			},
			// name,
			// paths,
			mods: Vec::new(),
			mods_errors: HashMap::new(),
			// executables: Vec::new(),
			mods_indexes: HashMap::new(),
			downloads: Vec::new(),
			// plugins: HashMap::new(),
			// override_config: None,
			// vfs_config: None,
			// game_identifier: GameIdentifier::default(),
		};

		new_instance.save()?;

		return Ok(new_instance);
	}

	pub fn save(&mut self) -> Result<(), String> {
		let instance_root_path = self.config.paths.root.clone();

		if !instance_root_path.exists() {
			std::fs::create_dir_all(&instance_root_path)
				.map_err(|e| format!("Failed to create instance root dir: {}", e.to_string()))?;
		}

		// Rebuild mod order
		self.rebuild_mods_order()?;

		let mut instance_clone = self.clone();
		instance_clone.mods_errors = HashMap::new();
		instance_clone.mods = vec![];

		// Deserialize the game instance to a value
		// let deserialized = match serde_json::to_value(&self.clone()) {
		// 	Ok(value) => value,
		// 	Err(e) => return Err(e.to_string()),
		// };

		// // Cast that value into the GameInstanceIO struct (for saving and loading)
		// let ioClone: GameInstanceIO = match serde_json::from_value(deserialized) {
		// 	Ok(io) => io,
		// 	Err(e) => return Err(e.to_string()),
		// };

		// Stringify the IO struct
		let json = serde_json::to_string(&instance_clone)
			.map_err(|e| format!("Failed to stringify instance: {}", e.to_string()))?;

		// Save the instance.json file
		crate::controllers::file_controller::save_file_with_backup(
			instance_root_path.join("instance.json"),
			json.as_bytes(),
		)
		.map_err(|e| format!("Failed to save instance: {}", e.to_string()))?;

		Ok(())
	}

	pub fn rebuild_mods_order(&mut self) -> Result<(), String> {
		// Build mod order (Name -> Index)
		let mut mods_indexes: HashMap<String, u32> = HashMap::new();

		for (index, mod_instance) in self.mods.iter().enumerate() {
			// Skip overwrite and base game
			if mod_instance.name == "overwrite" || mod_instance.name == "base" {
				continue;
			}
			mods_indexes.insert(mod_instance.name.clone(), index as u32);
		}

		// Inject game as first
		mods_indexes.insert(String::from("base"), 0);
		// Inject overwrite as last
		mods_indexes.insert(String::from("overwrite"), mods_indexes.len() as u32);

		self.mods_indexes = mods_indexes;

		Ok(())
	}

	// --------------------
	// Paths
	// --------------------

	pub fn instance_absolute_path(&self) -> PathBuf {
		return self.config.paths.root.clone();
	}

	pub fn parse_path_variables(&self, path: PathBuf) -> PathBuf {
		let mut path = path;

		if path.starts_with("$instance") {
			path = path.strip_prefix("$instance").unwrap().to_path_buf();
			// println!("path: {:?}", path);
			path = self.config.paths.root.join(path);
		}

		if path.starts_with("$game") {
			path = path.strip_prefix("$game").unwrap().to_path_buf();
			path = self.config.paths.game.join(path);
		}

		// if path.is_relative() {
		// 	path = self.paths.root.join(path);
		// }

		path
	}

	pub fn parse_string_variables(&self, mut input: String) -> String {
		// Replace $instance
		input = input.replace(
			"$instance",
			&self
				.config
				.paths
				.root
				.to_string_lossy()
				.trim_end_matches('/'),
		);

		// Replace $game
		input = input.replace(
			"$game",
			&self
				.config
				.paths
				.game
				.to_string_lossy()
				.trim_end_matches('/'),
		);

		return input;
	}

	pub fn get_mods_absolute_path(&self) -> PathBuf {
		return self.parse_path_variables(self.config.paths.internal.mods.clone());
	}

	pub fn get_deployment_mods_absolute_path(&self) -> PathBuf {
		return self.parse_path_variables(self.config.paths.deployment.mods.clone());
	}

	pub fn get_downloads_absolute_path(&self) -> PathBuf {
		return self.parse_path_variables(self.config.paths.internal.downloads.clone());
	}

	pub fn get_game_absolute_path(&self) -> PathBuf {
		return self.config.paths.game.clone();
	}

	pub fn overwrite_relative_path(&self) -> PathBuf {
		return self.get_mods_absolute_path().join("overwrite");
	}

	pub fn get_mods_deployment_file_structure(&self) -> Result<Vec<FileStructureSegment>, String> {
		let file_structure = file_controller::list_file_structure_relatively(
			self.get_deployment_mods_absolute_path(),
		);

		return file_structure.map_err(|e| e.to_string());
	}

	// --------------------
	// Mods
	// --------------------

	pub fn create_empty_mod(
		&mut self,
		name: String,
		version: Option<String>,
	) -> Result<InstanceMod, String> {
		let mods_path = self.get_mods_absolute_path();

		let instance_mod = InstanceMod::new(mods_path, name.clone(), version, ModInfo::default())?;
		instance_mod.save()?;

		Ok(instance_mod)
	}

	pub fn create_mod_version(
		&mut self,
		name: String,
		version: String,
		info: ModInfo,
	) -> Result<InstanceMod, String> {
		// let mods_path = self.get_mods_absolute_path();

		if name.len() == 0 {
			return Err("Mod name cannot be empty".to_string());
		}

		// Check if mod already exists
		let instance_mod = match self.get_mod_by_name(name.clone()) {
			Some(mod_instance) => mod_instance,
			None => {
				// Create new mod
				&mut self.create_empty_mod(name.clone(), None)?
			}
		};

		// Update info
		instance_mod.info = info;

		// Add version and save
		instance_mod.add_version(version)?;

		Ok(instance_mod.clone())
	}

	pub fn set_mod_active_version(
		&mut self,
		mod_name: String,
		version: String,
	) -> Result<(), String> {
		let instance_mod = match self.get_mod_by_name(mod_name.clone()) {
			Some(mod_instance) => mod_instance,
			None => {
				return Err(format!("Mod {} not found", mod_name));
			}
		};

		return instance_mod.set_active_version(version);
	}

	pub fn set_mod_enabled(&mut self, mod_name: String, enabled: bool) -> Result<(), String> {
		let instance_mod = match self.get_mod_by_name(mod_name.clone()) {
			Some(mod_instance) => mod_instance,
			None => {
				return Err(format!("Mod {} not found", mod_name));
			}
		};

		return instance_mod.set_enabled(enabled);
	}

	pub fn load_mods(&mut self) -> Result<Vec<InstanceMod>, String> {
		let mods_path = self.get_mods_absolute_path();

		if !mods_path.exists() {
			std::fs::create_dir_all(&mods_path).map_err(|e| e.to_string())?;
		}

		let mut mods = vec![];
		let mut errors: HashMap<String, String> = HashMap::new();

		for entry in std::fs::read_dir(mods_path).map_err(|e| e.to_string())? {
			let entry = entry.map_err(|e| e.to_string())?;
			let path = entry.path();

			// Check if the directory exists
			if !path.clone().is_dir() {
				continue;
			}

			// Ignore if overwrite or base folder
			if vec!["overwrite", "base"].contains(&path.file_name().unwrap().to_str().unwrap()) {
				continue;
			}

			// Check if it has an "mod.json" file
			if !path.clone().join("mod.json").exists() {
				errors.insert(
					path.clone()
						.file_name()
						.unwrap()
						.to_str()
						.unwrap()
						.to_string(),
					String::from("mod.json not found"),
				);
				continue;
			}

			// Attempt to load the mod from path
			match InstanceMod::load_from_path(path.clone()) {
				Ok(instanceMod) => {
					mods.push(instanceMod);
				}
				Err(err) => {
					errors.insert(
						path.clone().to_str().unwrap().to_string(),
						String::from(err),
					);
				}
			}
		}

		// Re-order mods using the mod order
		// If the index for a specific mod is not found, it will be placed at the end
		mods.sort_by(|a, b| {
			let a_index = self.mods_indexes.get(&a.name).unwrap_or(&u32::MAX);
			let b_index = self.mods_indexes.get(&b.name).unwrap_or(&u32::MAX);

			a_index.cmp(b_index)
		});

		// Inject the base folder
		mods.insert(
			0,
			InstanceMod {
				name: String::from("base"),
				enabled: true,
				absolute_path: self.get_deployment_mods_absolute_path(),
				versions: Vec::new(),
				selected_version_identifier: String::from("0.0.0"),
				info: ModInfo::default(),
			},
		);

		// Inject the overwrite folder
		mods.insert(
			mods.len(),
			InstanceMod {
				name: String::from("overwrite"),
				enabled: true,
				absolute_path: self.parse_path_variables(self.overwrite_relative_path()),
				versions: Vec::new(),
				selected_version_identifier: String::from("0.0.0"),
				info: ModInfo::default(),
			},
		);

		// Create overwrite folder if needed
		if !self.overwrite_relative_path().exists() {
			std::fs::create_dir_all(&self.overwrite_relative_path()).map_err(|e| e.to_string())?;
		}

		// Set mods in instance
		self.mods = mods.clone();
		self.mods_errors = errors.clone();

		// Rebuild the mod order
		self.rebuild_mods_order()?;

		Ok(mods)
	}

	pub fn open_mod_folder(&self, mod_name: String) -> Result<(), String> {
		let mods_path = self.get_mods_absolute_path();
		let mod_path = match mod_name == "base" {
			true => self.get_game_absolute_path(),
			false => mods_path.join(mod_name.clone()),
		};

		if !mod_path.exists() {
			return Err(format!("Mod with name \"{}\" does not exist", mod_name));
		}

		// Open mod folder
		crate::controllers::file_controller::open_folder(mod_path)?;

		Ok(())
	}

	pub fn move_mod_by_index(
		&mut self,
		mod_index: u32,
		mut target_index: u32,
		save: bool,
	) -> Result<(), String> {
		// Check if there are at least 2 mods
		if self.mods.len() < 2 {
			return Ok(());
		}

		// Ignore if same index
		if mod_index == target_index {
			return Ok(());
		}

		// Cannot be first, as base mod must always be first
		if target_index == 0 {
			target_index = 1;
		}

		// Cannot be last, as overwrite must always be last
		if target_index == self.mods.len() as u32 - 1 {
			target_index = self.mods.len() as u32 - 2;
		}

		// Move mod
		let mod_instance = self.mods.remove(mod_index as usize);

		// Check target index is not out of bounds
		let max_index = self.mods.len() as u32;
		if target_index > max_index {
			target_index = max_index;
		}

		// Insert mod at target index
		self.mods.insert(target_index as usize, mod_instance);

		// Reseat overwrite
		self.reseat_static_mods();

		// Rebuild mod order
		self.rebuild_mods_order()?;

		// Save instance (if needed)
		if save {
			self.save()?;
		}

		Ok(())
	}

	pub fn move_mods_by_indexes(
		&mut self,
		mut indexes: Vec<u32>,
		mut target_index: u32,
	) -> Result<Vec<u32>, String> {
		// Check if there are at least 2 mods
		if self.mods.len() < 2 {
			return Ok(indexes);
		}

		let last_index = self.mods.len() as u32 - 1;

		// If we have the last index in the list (overwrite), remove it
		if indexes.contains(&last_index) {
			indexes.retain(|x| *x != last_index);
		}

		// If we are targeting the last index, reduce by 1
		if target_index == last_index {
			target_index -= 1;
		}

		// Sort indexes in descending order
		let mut indexes = indexes;
		indexes.sort_by(|a, b| a.cmp(b));

		let mut temp_mods: Vec<InstanceMod> = vec![];

		// Remove mods from list
		for index in indexes.iter().rev() {
			if (*index as usize) >= self.mods.len() {
				println!("Index out of bounds: {}", index);
				continue;
			}

			let mod_instance = self.mods.remove(*index as usize);
			temp_mods.push(mod_instance);
		}

		// Check target index is not out of bounds
		let max_index = self.mods.len() as u32;
		let mut target_index = target_index;
		if target_index > max_index {
			target_index = max_index;
		}

		let mut new_indexes: Vec<u32> = vec![];

		// Insert mods at target index
		for (index, mod_instance) in temp_mods.iter().rev().enumerate() {
			let new_index = target_index + index as u32;
			self.mods.insert((new_index) as usize, mod_instance.clone());
			new_indexes.push(new_index);
		}

		// Reseat overwrite
		self.reseat_static_mods();

		// Save instance
		self.save()?;

		Ok(new_indexes)
	}

	pub fn move_mod_by_name(&mut self, mod_name: String, target_index: u32) -> Result<(), String> {
		let mod_index = self
			.mods_indexes
			.get(&mod_name)
			.ok_or(format!("Mod with name \"{}\" does not exist", mod_name))?;

		return self.move_mod_by_index(*mod_index, target_index, true);
	}

	pub fn reseat_static_mods(&mut self) {
		let base_index = self
			.mods
			.iter()
			.position(|mod_instance| mod_instance.name == "base");

		let base_mod = self.mods.remove(base_index.expect("base mod not found"));

		self.mods.insert(0, base_mod);

		let overwrite_index = self
			.mods
			.iter()
			.position(|mod_instance| mod_instance.name == "overwrite");

		let overwrite = self
			.mods
			.remove(overwrite_index.expect("Overwrite not found"));

		self.mods.push(overwrite);
	}

	pub fn get_enabled_mods(&mut self) -> Vec<&mut InstanceMod> {
		return self
			.mods
			.iter_mut()
			.filter(|mod_instance| mod_instance.enabled)
			.collect();
	}

	pub fn get_mod_by_name(&mut self, mod_name: String) -> Option<&mut InstanceMod> {
		self.mods
			.iter_mut()
			.find(|mod_instance| mod_instance.name == mod_name)
	}

	pub fn delete_mod(&mut self, mod_name: String) -> Result<(), String> {
		let mods_absolute_path = self.get_mods_absolute_path();

		let instance_mod = match self.get_mod_by_name(mod_name.clone()) {
			Some(instance_mod) => instance_mod,
			None => {
				return Err(format!("Mod with name \"{}\" does not exist", mod_name));
			}
		};

		// Delete mod
		instance_mod.delete_mod(mods_absolute_path)?;

		// Remove mod from list
		self.mods
			.retain(|mod_instance| mod_instance.name != mod_name);

		// Rebuild mod order
		self.rebuild_mods_order()?;

		// Save instance
		return self.save();
	}

	pub fn delete_mod_version(
		&mut self,
		mod_name: String,
		mod_version: Option<String>,
	) -> Result<(), String> {
		let mods_absolute_path = self.get_mods_absolute_path();

		// Delete from filesystem
		let instance_mod = match self.get_mod_by_name(mod_name.clone()) {
			Some(instance_mod) => instance_mod,
			None => {
				return Err(format!("Mod with name \"{}\" does not exist", mod_name));
			}
		};

		match mod_version {
			Some(mod_version) => {
				instance_mod.delete_version(mod_version)?;
			}
			None => {
				instance_mod.delete_mod(mods_absolute_path)?;
			}
		}

		return Ok(());
	}

	// --------------------
	// Plugins
	// --------------------
	pub fn get_plugins(&mut self) -> Result<HashMap<String, Vec<BethesdaPlugin>>, String> {
		let mut plugins: HashMap<String, Vec<BethesdaPlugin>> = HashMap::new();

		let game_identifier = self.config.game_identifier.clone();

		for enabled_mod in self.get_enabled_mods() {
			let mod_plugins = enabled_mod.get_plugins(game_identifier.clone())?;
			plugins.insert(enabled_mod.name.clone(), mod_plugins);
		}

		return Ok(plugins);
	}

	pub fn get_load_order(&self) -> Result<(), String> {
		let game_settings = plugin_controller::read_load_order(
			self.config.game_identifier,
			self.get_game_absolute_path().as_path(),
			self.parse_path_variables(self.config.paths.internal.settings.clone())
				.as_path(),
			self.parse_path_variables(self.config.paths.internal.saves.clone()),
		)?;

		let mut load_order = game_settings.into_load_order();
		load_order
			.load()
			.map_err(|err| format!("Failed to load load order: {}", err.to_string()))?;

		println!("Plugin Count 1: {:?}", load_order.plugin_names().len());

		load_order
			.add("test-1")
			.map_err(|err| format!("Failed to add to load order: {}", err.to_string()))?;

		println!("Plugin Count 2: {:?}", load_order.plugin_names().len());

		return Ok(());
	}

	// --------------------
	// Config
	// --------------------

	pub fn update_config(&mut self, config: GameInstanceConfig) -> Result<(), String> {
		self.config = config;

		self.save()?;

		Ok(())
	}

	// --------------------
	// Downloads
	// --------------------

	pub async fn start_downloads(
		&mut self,
		state_mutex: Arc<Mutex<ApplicationState>>,
	) -> Result<(), String> {
		println!("Starting downloads...");

		let concurrent_downloads = self.config.downloads_config.concurrent_downloads;
		let num_threads = self.config.downloads_config.threads_per_download;

		let currently_downloading_count = self
			.downloads
			.iter()
			.filter(|d| {
				d.downloader.is_some()
					|| d.status == downloader::DownloadStatus::Downloading
					|| d.status == downloader::DownloadStatus::Merging
					|| d.status == downloader::DownloadStatus::Verifying
			})
			.count();

		let mut clone_downloads = self.clone().downloads;
		let pending_downloads = clone_downloads.iter_mut().filter(|d| {
			d.downloader.is_none()
				&& d.status != downloader::DownloadStatus::Downloaded
				&& d.status != downloader::DownloadStatus::Downloading
				&& d.status != downloader::DownloadStatus::Merging
				&& d.status != downloader::DownloadStatus::Verifying
		});

		// How many downloads can we start?
		// (Desired - Downloading) -> (Concurrent Downloads - Currently Downloading)
		let take_count = concurrent_downloads - currently_downloading_count;
		if take_count > 0 {
			println!("Starting {} downloads", take_count);

			// Start downloads
			for download in pending_downloads.take(take_count) {
				let download_path = self.get_downloads_absolute_path();

				download.start(state_mutex.clone(), download_path, num_threads);
			}
		}

		return Ok(());
	}

	// --------------------
	// Executables
	// --------------------

	pub fn set_executables(&mut self, executables: Vec<InstanceExecutable>) -> Result<(), String> {
		// Set executables
		self.config.executables = executables
			.iter()
			.map(|e| {
				let mut e = e.clone();

				// Fetch executable icon, if necessary
				if e.path.is_some() && (e.icon.is_none() || e.icon == Some("".to_string())) {
					e.icon = self.get_executable_icon_base64(&e);
				}

				return e;
			})
			.collect();

		// TODO: Load other executables data

		// Save config
		self.save()?;

		return Ok(());
	}

	fn get_executable_icon_base64(&self, executable: &InstanceExecutable) -> Option<String> {
		if executable.path.is_none() {
			return None;
		}

		let executable_relative_path = executable.path.as_ref().unwrap().to_owned();
		let executable_absolute_path = self.parse_path_variables(executable_relative_path.clone());

		if !file_controller::file_exists(&executable_absolute_path) {
			return None;
		}

		// Save icon in /tmp
		let mut ico_output_path = PathBuf::from("/tmp/rusty-mod-manager");
		match file_controller::create_folder(&ico_output_path) {
			Ok(_) => {}
			Err(err) => {
				eprintln!("Failed to create '/tmp/rusty-mod-manager': {}", err);
				return None;
			}
		}

		// Generate a random string in case we fail to get the executable name somehow
		let rand_string: String = rand::thread_rng()
			.sample_iter(&Alphanumeric)
			.take(7)
			.map(char::from)
			.collect();

		// Append the executable name or a random string
		ico_output_path.push(
			executable_relative_path
				.file_name()
				.unwrap_or(OsStr::new(&rand_string))
				.to_string_lossy()
				.to_string(),
		);

		let ico_output_path_with_ico = ico_output_path.clone().with_extension("ico");

		// Run wrestool to extract icon
		let output_arg = &format!("--output={}", ico_output_path_with_ico.to_str()?);
		let output = match Command::new("wrestool")
			.env("LC_ALL", "C")
			.args(&["-x", "-t14", output_arg, executable_absolute_path.to_str()?])
			.output()
		{
			Ok(output) => output,
			Err(err) => {
				eprintln!("Failed to run 'wrestool' (Is it installed?): {}", err);
				return None;
			}
		};

		if !output.status.success() || !output.stderr.is_empty() {
			let error_message = String::from_utf8_lossy(&output.stderr);
			eprintln!("Failed to extract icon from executable: {}", error_message);
			return None;
		}

		// println!("Extracted icon: {} -> {}", executable_absolute_path.clone().to_string_lossy(), ico_output_path_with_ico.clone().to_string_lossy());

		let file = std::fs::File::open(ico_output_path_with_ico).expect("Failed to open file");

		let icon_dir = match ico::IconDir::read(file) {
			Ok(icon_dir) => icon_dir,
			Err(_) => {
				println!(
					"Failed to read icon dir for: {}",
					executable_relative_path.clone().to_string_lossy()
				);
				return None;
			}
		};

		let highest_entry: &ico::IconDirEntry = icon_dir
			.entries()
			.iter()
			.max_by(|a, b| a.width().cmp(&b.width()))
			.unwrap();

		let image_icon = highest_entry.decode().unwrap();

		// Create an `ImageBuffer` from the raw RGBA data
		let image_buffer = match ImageBuffer::from_raw(
			highest_entry.width(),
			highest_entry.height(),
			image_icon.rgba_data().to_owned(),
		) {
			Some(image_buffer) => image_buffer,
			None => {
				println!(
					"Failed to create image buffer for: {}",
					executable_relative_path.clone().to_string_lossy()
				);
				return None;
			}
		};

		let dynamic_image: DynamicImage = image::DynamicImage::ImageRgba8(image_buffer).into();
		let mut bytes: Vec<u8> = Vec::new();
		let result = dynamic_image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png);

		if result.is_err() {
			println!(
				"Failed to write image to bytes for: {}",
				executable_relative_path.to_string_lossy()
			);
			return None;
		}

		let base64 = general_purpose::STANDARD.encode(bytes);
		return Some(base64);
	}

	pub fn run_executable(&self, executable: InstanceExecutable) -> Result<Child, String> {
		let mut main_command;

		if executable.command.is_some() {
			main_command = executable.command.unwrap();
		} else if executable.path.is_some() {
			main_command = executable.path.unwrap().to_string_lossy().to_string();
		} else {
			return Err("No command or executable specified".to_string());
		}

		match executable.args {
			Some(args) => {
				main_command = format!("{} {}", main_command, args);
			}
			None => {}
		};

		let main_command_parsed = self.parse_string_variables(main_command);

		let mut binding = std::process::Command::new("sh");
		let process_command = binding.arg("-c").arg(main_command_parsed.clone());

		println!("Running executable: {}", main_command_parsed);

		return match process_command.spawn() {
			Ok(child) => Ok(child),
			Err(err) => Err(err.to_string()),
		};
	}

	// --------------------
	// Virtual File-System
	// --------------------

	pub fn mount_vfs(
		&self,
		fallback_vfs_config: VFSConfig,
	) -> Result<Vec<Box<dyn BaseVFS>>, String> {
		let vfs_config = match self.config.vfs_config.clone() {
			Some(instance_vfs_config) => instance_vfs_config,
			None => fallback_vfs_config,
		};

		let mut return_vfs_vec: Vec<Box<dyn BaseVFS>> = Vec::new();

		// First, mount mods

		// Get mods, filter by enabled
		let filtered_mods: Vec<&InstanceMod> = self
			.mods
			.iter()
			.filter(|mod_source| {
				mod_source.enabled
					&& mod_source.versions.len() > 0
					&& !vec!["overwrite", "base"].contains(&mod_source.name.as_str())
			})
			.collect();

		let mods_sources = filtered_mods
			.iter()
			.map(|instance_mod| instance_mod.get_selected_version_absolute_path().clone())
			.collect();

		let mods_mount_paths = VFSMountPaths {
			target: self.parse_path_variables(self.config.paths.deployment.mods.clone()),
			sources: mods_sources,
			overwrite: self.parse_path_variables(self.overwrite_relative_path()),
			// TODO: Define workpath as a variable in VFS Config
			workdir: self
				.instance_absolute_path()
				.join(".vfs_workdir")
				.join("mods"),
		};

		let vfs_mods = self
			.mount_vfs_sub("mods", vfs_config.clone(), mods_mount_paths, true)
			.map_err(|e| format!("Failed to mount sub-vfs ({}): {}", "mods", e.to_string()))?;

		return_vfs_vec.push(vfs_mods);

		// Mount Saves
		match &self.config.paths.deployment.saves {
			Some(save_deployment_path) => {
				if !save_deployment_path.is_dir() {
					return Err("Saves deployment path is not a directory".to_string());
				}

				// Instance's internal save path
				let internal_save_path =
					self.parse_path_variables(self.config.paths.internal.saves.clone());

				// Create folder if needed
				file_controller::create_folder(&internal_save_path).map_err(|e| {
					format!(
						"Failed to create instance internal saves folder: {}",
						e.to_string()
					)
				})?;

				let saves_mount_paths = VFSMountPaths {
					target: self.parse_path_variables(save_deployment_path.clone()),
					sources: Vec::new(),
					overwrite: internal_save_path,
					// TODO: Define workpath as a variable in VFS Config
					workdir: self
						.instance_absolute_path()
						.join(".vfs_workdir")
						.join("saves"),
				};

				let vfs_saves = self
					.mount_vfs_sub("saves", vfs_config.clone(), saves_mount_paths, false)
					.map_err(|e| {
						format!("Failed to mount sub-vfs ({}): {}", "saves", e.to_string())
					})?;

				return_vfs_vec.push(vfs_saves);
			}
			None => {}
		}

		// Mount Settings
		match &self.config.paths.deployment.settings {
			Some(settings_deployment_path) => {
				if !settings_deployment_path.is_dir() {
					return Err("Settings deployment path is not a directory".to_string());
				}

				// Instance's internal save path
				let internal_settings_path =
					self.parse_path_variables(self.config.paths.internal.settings.clone());

				// Create folder if needed
				file_controller::create_folder(&internal_settings_path).map_err(|e| {
					format!(
						"Failed to create instance internal settings folder: {}",
						e.to_string()
					)
				})?;

				let settings_mount_paths = VFSMountPaths {
					target: self.parse_path_variables(settings_deployment_path.clone()),
					sources: vec![internal_settings_path],
					overwrite: self.parse_path_variables(self.overwrite_relative_path()),
					// TODO: Define workpath as a variable in VFS Config
					workdir: self
						.instance_absolute_path()
						.join(".vfs_workdir")
						.join("settings"),
				};

				let vfs_settings = self
					.mount_vfs_sub("settings", vfs_config.clone(), settings_mount_paths, false)
					.map_err(|e| {
						format!(
							"Failed to mount sub-vfs ({}): {}",
							"settings",
							e.to_string()
						)
					})?;

				return_vfs_vec.push(vfs_settings);
			}
			None => {}
		}

		return Ok(return_vfs_vec);
	}

	fn mount_vfs_sub(
		&self,
		name: &str,
		vfs_config: VFSConfig,
		mount_paths: VFSMountPaths,
		should_overlay_target: bool,
	) -> Result<Box<dyn BaseVFS>, String> {
		let mount_name = format!("{}{}", self.config.name.clone(), name);

		let vfs_mount_config = VFSMountConfig {
			mount_name,
			command: vfs_config.command,
			paths: mount_paths,
			should_overlay_target,
		};

		let vfs_implementation = match vfs_config.implementation {
			VFSImplementation::UnionFSFuse => UnionFSFuse {
				config: vfs_mount_config,
			},
			// TODO: Implement OverlayFS
			VFSImplementation::OverlayFS => UnionFSFuse {
				config: vfs_mount_config,
			},
			// VFSImplementation::OverlayFS => OverlayFS::new(vfs_config),
		};

		vfs_implementation.mount()?;

		return Ok(Box::new(vfs_implementation));
	}

	// pub fn unmount_vfs(&self, fallback_vfs_config: VFSConfig) -> Result<(), String> {
	// 	let vfs_config = match self.config.vfs_config.clone() {
	// 		Some(instance_vfs_config) => instance_vfs_config,
	// 		None => fallback_vfs_config,
	// 	};
	// }
}
