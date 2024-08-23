use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq, Default, Copy)]
pub enum VFSImplementation {
	#[default]
	UnionFSFuse,
	OverlayFS,
}

#[derive(Default, Debug)]
#[taurpc::ipc_type]
pub struct VFSConfig {
	#[serde(default)]
	pub implementation: VFSImplementation,
	#[serde(default)]
	pub command: Option<String>,
}

impl VFSConfig {
	pub fn new() -> Self {
		return Self {
			implementation: Default::default(),
			command: None,
		};
	}
}
