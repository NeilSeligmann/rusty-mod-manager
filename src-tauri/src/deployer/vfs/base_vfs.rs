use std::path::PathBuf;

// use erased_serde::serialize_trait_object;

#[taurpc::ipc_type]
pub struct VFSMountPaths {
	// Target to virtualize
	pub target: PathBuf,
	// Sources to combine with target
	pub sources: Vec<PathBuf>,
	// Where to write changes (copy-on-write)
	pub overwrite: PathBuf,
	// Temporal working directory
	pub workdir: PathBuf,
}

#[taurpc::ipc_type]
pub struct VFSMountConfig {
	pub mount_name: String,
	pub command: Option<String>,
	pub paths: VFSMountPaths,
	#[serde(skip)]
	pub should_overlay_target: bool,
}

#[typetag::serde(tag = "type")]
pub trait BaseVFS {
	fn set_config(&mut self, config: VFSMountConfig) -> Result<(), String>;
	fn mount(&self) -> Result<(), String>;
	fn unmount(&self) -> Result<(), String>;
}
