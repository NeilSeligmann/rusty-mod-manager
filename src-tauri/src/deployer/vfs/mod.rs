use self::base_vfs::{BaseVFS, VFSMountConfig};

pub mod base_vfs;
pub mod union_fs;
pub mod union_fs_fuse;

// pub enum VFSImplementation {
// 	UnionFS,
// }

// pub fn get_vfs(implementation: VFSImplementation, config: VFSMountConfig) -> Box<dyn BaseVFS> {
// 	match implementation {
// 		VFSImplementation::UnionFS => Box::new(union_fs_fuse::UnionFSFuse { config }),
// 	}
// }
