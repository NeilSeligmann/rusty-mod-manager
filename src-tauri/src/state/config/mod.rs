use std::path::PathBuf;
use nexusmods_config::NexusModsConfig;

pub mod vfs_config;
pub mod nexusmods_config;

fn _default_nexusmods_config() -> NexusModsConfig {
	NexusModsConfig::new()
}

fn _default_vector_empty<T>() -> Vec<T> {
	Vec::new()
}

fn _default_vfs_config() -> vfs_config::VFSConfig {
	vfs_config::VFSConfig::new()
}

#[taurpc::ipc_type]
pub struct ApplicationConfig {
	#[serde(default = "_default_vector_empty")]
	pub available_instances_paths: Vec<PathBuf>,
	// #[serde(default = "_default_nexusmods_config")]
	pub nexusmods: NexusModsConfig,
	// #[serde(default= "_default_vfs_config")]
	#[serde(default)]
	pub default_vfs_config: vfs_config::VFSConfig,
}

impl ApplicationConfig {
	pub fn new() -> Self {
		return Self {
			available_instances_paths: Vec::new(),
			nexusmods: NexusModsConfig::new(),
			default_vfs_config: vfs_config::VFSConfig::new(),
		};
	}
}
