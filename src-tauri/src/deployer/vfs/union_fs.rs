use serde::{Deserialize, Serialize};

use super::base_vfs::{BaseVFS, VFSMountConfig, VFSMountPaths};
use std::{
	process::{Child, Command},
	thread,
	time::Duration,
};

#[derive(Serialize, Deserialize)]
pub struct UnionFS {
	pub config: VFSMountConfig,
}

impl UnionFS {}

#[typetag::serde]
impl BaseVFS for UnionFS {
	fn set_config(&mut self, config: VFSMountConfig) -> Result<(), String> {
		self.config = config.clone();
		return Ok(());
	}

	fn mount(&self) -> Result<(), String> {
		// Use provided command, if any
		let command = match self.config.command.clone() {
			Some(command) => command,
			None => String::from("unionfs"),
		};

		// Validate command is not an empty string
		if command.len() <= 0 {
			return Err(String::from(
				"Invalid command provided, cannot be empty string!",
			));
		}

		// Validate command exists
		let check_command = Command::new("which").arg(command.clone()).output();
		match check_command {
			Ok(result) => {
				if (result.stderr.len() > 0) || (result.stdout.len() == 0) {
					return Err(String::from(format!(
						"Command \"{}\" not found, is it installed?",
						command
					)));
				}
			}
			Err(_) => {
				return Err(String::from(format!(
					"Failed to run the check for a valid command: \"{}\"",
					command
				)));
			}
		}

		// Folder "dance" needed for Union FS - Fuse
		// 1) Change the name of the original mod folder (Ex. "Data") to UNION_FS_TEMP_FOLDER
		// 2) Create a new folder with the original name of the folder we just moved
		// 3) Use the old folder as a layer, and the new folder as the mount point

		// Parsed paths/layers for UnionFS-Fuse
		let mut parsed_layers: String = String::from("");

		// Top layer is Overwrite folder, in RW mode
		parsed_layers
			.push_str(format!("{}=RW", self.config.paths.overwrite.to_str().unwrap()).as_str());

		// Lower layers are the mods folders, in RO mode
		for path in &self.config.paths.sources {
			if !path.exists() {
				println!("Path {:?} does not exist", path);
				continue;
			}
			parsed_layers.push_str(format!(":{}=RO", path.to_str().unwrap()).as_str());
		}

		// Push the mount folder as the last layer, in RO mode
		parsed_layers
			.push_str(format!(":{}=RO", self.config.paths.target.to_str().unwrap()).as_str());

		println!("Mounting UnionFS with the following parameters:");
		println!("Mount path: {:?}", self.config.paths.target);
		println!("Work path: {:?}", self.config.paths.workdir);
		println!("Parsed Paths: {:?}", parsed_layers);
		println!("Mount exists: {:?}", self.config.paths.target.exists());

		let result = Command::new(command)
			.arg("-o")
			.arg("cow")
			.arg("-o")
			// TODO: We way need to increase max files
			.arg("max_files=32768")
			.arg("-o")
			.arg("hide_meta_files")
			// .arg("-o")
			// .arg("allow_other")
			// .arg("-o")
			// .arg("use_ino")
			// .arg("-o")
			// .arg("nonempty")
			// .arg("-o")
			// .arg("suid")
			// .arg(parsed_paths)
			.arg("-o")
			.arg(format!("dirs={}", parsed_layers))
			.arg(self.config.paths.target.to_str().unwrap())
			.output();

		println!("UnionFS-Fuse Result: {:?}", result);

		match result {
			Ok(output) => {
				if output.stderr.is_empty() {
					return Ok(());
				}

				return Err(String::from_utf8(output.stderr).unwrap());
			}
			Err(e) => {
				println!("Error running UnionFS-Fuse: {:?}", e);
				return Err(String::from(format!("Error running UnionFS-Fuse: {:?}", e)));
			}
		}
	}

	fn unmount(&self) -> Result<(), String> {
		let result = Command::new("umount")
			.arg(self.config.paths.target.clone())
			.spawn();

		// Check if command ran succesfully
		match result {
			Ok(mut child) => match child.wait() {
				Ok(status) => {
					if !status.success() {
						return Err(format!("Failed to unmount: {:?}", status));
					}
				}
				Err(e) => {
					return Err(format!("Failed to wait for the unmount process: {:?}", e));
				}
			},
			Err(e) => {
				return Err(format!("Failed to spawn unmount process: {:?}", e));
			}
		}

		return Ok(());
	}
}
