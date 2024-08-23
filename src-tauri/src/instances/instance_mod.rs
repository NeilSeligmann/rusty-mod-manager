use crate::controllers::plugin_controller::BethesdaPlugin;
use crate::controllers::{file_controller, plugin_controller};
use std::collections::HashMap;
use std::{
	ffi::OsString,
	path::{Path, PathBuf},
};

use super::GameIdentifier;

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ModInfo {
	// pub name: String,
	pub author: Option<String>,
	// pub version: Option<String>,
	pub website: Option<String>,
	pub description: Option<String>,
	pub categories: Vec<String>,
}

impl Default for ModInfo {
	fn default() -> Self {
		Self {
			author: None,
			website: None,
			description: None,
			categories: vec![],
		}
	}
}

// #[taurpc::ipc_type]
// #[derive(Debug)]
// pub struct InstanceModVersion {
// 	// Version identifier, unique per instance mod
// 	pub identifier: String,
// }

// impl PartialEq<InstanceModVersion> for InstanceModVersion {
// 	fn eq(&self, other: &InstanceModVersion) -> bool {
// 		self.identifier == *other.identifier
// 	}
// }

// impl InstanceModVersion {
// 	// pub fn save(&self, mod_path: PathBuf) -> Result<(), String> {
// 	// 	let absolute_path = mod_path.join(&self.identifier);

// 	// 	let json = serde_json::to_string(&self).map_err(|e| e.to_string())?;

// 	// 	file_controller::save_file(
// 	// 		absolute_path.join("modVersion.json"),
// 	// 		json.as_bytes(),
// 	// 	)
// 	// 	.map_err(|e| e.to_string())?;

// 	// 	Ok(())
// 	// }
// }

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct InstanceMod {
	// Mod full, absolute path
	#[serde(skip)]
	pub absolute_path: PathBuf,
	// Mod name (independent of folder name)
	pub name: String,
	// Vector of versions
	pub versions: Vec<String>,
	// pub versions: HashMap<String, InstanceModVersion>,
	// Unique name of the selected version
	pub selected_version_identifier: String,
	// Whether the mod is enabled (if disabled, it won't be deployed)
	pub enabled: bool,
	// Mod info (author, website, etc)
	pub info: ModInfo,
}

impl InstanceMod {
	pub fn get_versions_path(&self) -> PathBuf {
		self.absolute_path.join("versions")
	}

	pub fn new(
		mut location: PathBuf,
		name: String,
		version: Option<String>,
		info: ModInfo,
	) -> Result<Self, String> {
		location.push(name.clone());

		let mut instanceMod = InstanceMod {
			absolute_path: location,
			name,
			versions: Vec::new(),
			selected_version_identifier: version.clone().unwrap_or(String::from("invalid")),
			enabled: true,
			info,
		};

		// Save mod
		instanceMod.save()?;

		// Save mod version
		if version.is_some() {
			instanceMod.add_version(version.unwrap())?;
		}

		return Ok(instanceMod);
	}

	pub fn load_from_path(path: PathBuf) -> Result<Self, String> {
		let json = match file_controller::read_file(path.join("mod.json")) {
			Ok(json) => json,
			Err(e) => return Err(e.to_string()),
		};

		let mut instanceMod: InstanceMod = serde_json::from_str(&json)
			.map_err(|e| format!("Failed to parse instanceMod from json: {}", e.to_string()))?;

		instanceMod.absolute_path = path;

		// Load versions
		// TODO: Load versions from path, in case they were added/deleted

		Ok(instanceMod)
	}

	pub fn save(&self) -> Result<(), String> {
		println!("Saving mod: {:?}", self.absolute_path);
		let json = serde_json::to_string(&self)
			.map_err(|e| format!("Failed to serialize instanceMod: {}", e.to_string()))?;

		file_controller::save_file_with_backup(
			self.absolute_path.join("mod.json"),
			json.as_bytes(),
		)
		.map_err(|e| format!("Failed to save instanceMod: {}", e.to_string()))?;

		Ok(())
	}

	pub fn get_version_absolute_path(&self, version_identifier: String) -> PathBuf {
		// Overwrite has no versions, so just return the absolute path
		if self.name == "overwrite" || self.name == "base" {
			return self.absolute_path.clone();
		}

		self.get_versions_path().join(version_identifier)
	}

	pub fn get_selected_version_absolute_path(&self) -> PathBuf {
		self.get_version_absolute_path(self.selected_version_identifier.clone())
	}

	pub fn delete_mod(&mut self, mods_path: PathBuf) -> Result<(), String> {
		return file_controller::delete_folder_safe(self.absolute_path.clone(), mods_path)
			.map_err(|e| format!("Failed to delete instanceMod: {}", e.to_string()));
	}

	pub fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
		self.enabled = enabled;
		self.save()?;
		Ok(())
	}

	pub fn set_active_version(&mut self, version_identifier: String) -> Result<(), String> {
		if !self.has_version(version_identifier.clone()) {
			return Err("Version not found!".to_string());
		}
		self.selected_version_identifier = version_identifier.clone();
		self.save()?;
		Ok(())
	}

	pub fn add_version(&mut self, version_identifier: String) -> Result<(), String> {
		if self.has_version(version_identifier.clone()) {
			return Err("Version already exists!".to_string());
		}

		self.versions.push(version_identifier.clone());

		// Set active version and save
		self.set_active_version(version_identifier)?;

		return Ok(());
	}

	pub fn has_version(&self, version_identifier: String) -> bool {
		return self.versions.contains(&version_identifier);
	}

	pub fn delete_version(&mut self, version_identifier: String) -> Result<(), String> {
		if !self.has_version(version_identifier.clone()) {
			return Err("Cannot delete inexistent version.".to_string());
		}

		// Remove version from vector
		self.versions.retain(|v| v != &version_identifier);

		// Delete folder version
		file_controller::delete_folder_safe(
			self.get_version_absolute_path(version_identifier),
			self.get_versions_path(),
		)
		.map_err(|e| format!("Failed to delete version: {}", e.to_string()))?;

		return self.save();
	}

	pub fn get_plugins(
		&mut self,
		game_identifier: GameIdentifier,
	) -> Result<Vec<BethesdaPlugin>, String> {
		let mut plugins: Vec<BethesdaPlugin> = Vec::new();

		let version_absolute_path = self.get_selected_version_absolute_path();
		let plugins_in_mod = file_controller::get_files_in_folder_with_extensions(
			version_absolute_path,
			vec!["esl", "esm", "esp"],
		);

		for plugin_path in plugins_in_mod {
			plugins.push(plugin_controller::read_plugin(
				game_identifier.clone(),
				plugin_path.as_path(),
			)?);
		}

		return Ok(plugins);
	}
}
