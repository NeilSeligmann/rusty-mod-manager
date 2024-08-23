use crate::{controllers::file_controller, state};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::command;

use super::base_vfs::{BaseVFS, VFSMountConfig, VFSMountPaths};
use std::{
	path::{Path, PathBuf},
	process::{Child, Command},
	thread,
	time::Duration,
};

// static OFS_PREFIX: &str = "rmm-overlay-";
static UNION_FS_TEMP_FOLDER_PREFIX: &str = ".rmm-temp-";
static VERSION_REGEX: &str = "unionfs-fuse version: ([0-9\\.]{1,10})";

#[derive(Serialize, Deserialize)]
pub struct UnionFSFuse {
	pub config: VFSMountConfig,
}

impl UnionFSFuse {
	fn get_temp_mount_folder(&self) -> PathBuf {
		let mount = self.config.paths.target.clone();

		let mut mount_temp = mount.clone();

		// Get last segment of mount path
		let mount_clone = mount.clone();
		let mount_last_segment = mount_clone
			.file_name()
			.expect("Unable to extract foldername of mount path!");

		// Remove last segment from mount path
		mount_temp.pop();

		// Add last segment back with prefix
		mount_temp.push(Path::new(
			format!(
				"{}{}",
				UNION_FS_TEMP_FOLDER_PREFIX,
				mount_last_segment.to_str().unwrap()
			)
			.as_str(),
		));

		return mount_temp;
	}

	fn move_mount_folder(&self) -> Result<(), String> {
		let mount = self.config.paths.target.clone();

		let mount_temp = self.get_temp_mount_folder();

		println!(
			"Moving mount folder \"{:?}\" to \"{:?}\"",
			mount, mount_temp
		);

		match file_controller::move_folder(mount.clone(), mount_temp.clone()) {
			Ok(_) => {}
			Err(e) => {
				println!("Error moving folder: {:?}", e);
				return Err(String::from("Error moving folder"));
			}
		}

		// Wait for a bit to make sure the folder is moved
		thread::sleep(Duration::from_millis(300));

		// We now need to create the mount folder again (as we previously moved it)
		match file_controller::create_folder(&mount) {
			Ok(_) => {}
			Err(e) => {
				println!("Error creating mount folder at \"{:?}\"", e);
				return Err(String::from(format!(
					"Error creating mount folder: {:?}",
					e
				)));
			}
		}

		// Wait for a bit to make sure the temp folder is created
		thread::sleep(Duration::from_millis(300));
		return Ok(());
	}

	fn restore_mount_folder(&self) -> Result<(), String> {
		let mount = self.config.paths.target.clone();

		let mount_temp = self.get_temp_mount_folder();

		// Delete the mount folder that we previously created
		match file_controller::delete_folder_if_empty(mount.clone()) {
			Ok(_) => {}
			Err(e) => {
				println!("Error deleting mount folder at \"{:?}\", is it empty?", e);
				return Err(String::from("Error deleting mount folder"));
			}
		}

		println!(
			"Restoring mount folder \"{:?}\" to \"{:?}\"",
			mount_temp, mount
		);

		// Restore the mount folder
		match file_controller::move_folder(mount_temp.clone(), mount.clone()) {
			Ok(_) => {}
			Err(e) => {
				println!("Error moving folder: {:?}", e);
				return Err(String::from("Error moving folder"));
			}
		}

		return Ok(());
	}
}

#[typetag::serde]
impl BaseVFS for UnionFSFuse {
	fn set_config(&mut self, config: VFSMountConfig) -> Result<(), String> {
		self.config = config.clone();
		return Ok(());
	}

	fn mount(&self) -> Result<(), String> {
		// Use provided command, if any
		let mut command = match self.config.command.clone() {
			Some(command) => command,
			None => String::from(""),
		};

		// Validate command is not an empty string
		if command.len() <= 0 {
			// If a command was not provided, check to see if we find "unionfs" or "unionfs-fuse"
			let unionfs_output = Command::new("unionfs")
				.arg("--version")
				.output()
				.expect("Failed to run the check for a valid command (unionfs)");

			// Did the command succeed?
			if unionfs_output.stderr.len() < 1 {
				// Then we use "unionfs" as our command
				command = String::from("unionfs");
			} else {
				// "unionfs" failed, lets check "unionfs-fuse"
				let unionfs_fuse_output = Command::new("unionfs-fuse")
					.arg("--version")
					.output()
					.expect("Failed to run the check for a valid command (unionfs-fuse)");

				// Did the command succeed?
				if unionfs_fuse_output.stderr.len() < 1 {
					// Then we use "unionfs-fuse" as our command
					command = String::from("unionfs-fuse");
				}
			}

			if command.len() < 1 {
				return Err(String::from(
					"No command provided and neither of \"unionfs\" or \"unionfs-fuse\" were found.",
				));
			}
		}

		// Validate command exists
		// let check_command = Command::new("which").arg(command.clone()).output().expect("Failed to run the check for a valid command");
		// if (result.stderr.len() > 0) || (result.stdout.len() == 0) {
		// 	return Err(String::from(format!(
		// 		"Command \"{}\" not found, is it installed?",
		// 		command
		// 	)));
		// }
		// match check_command {
		// 	Ok(result) => {
		// 		if (result.stderr.len() > 0) || (result.stdout.len() == 0) {
		// 			return Err(String::from(format!(
		// 				"Command \"{}\" not found, is it installed?",
		// 				command
		// 			)));
		// 		}
		// 	}
		// 	Err(_) => {
		// 		return Err(String::from(format!(
		// 			"Failed to run the check for a valid command: \"{}\"",
		// 			command
		// 		)));
		// 	}
		// }

		// Validate the unionfs-fuse version
		let validate_unionfs_version = Command::new(command.clone()).arg("--version").output();
		match validate_unionfs_version {
			Ok(result) => {
				if (result.stderr.len() > 0) || (result.stdout.clone().len() == 0) {
					return Err(String::from(format!(
						"Failed to run: \"{} --version\"",
						command
					)));
				}

				let output_string = String::from_utf8(result.stdout.clone()).unwrap();

				// Get version using regex
				let version = match regex::Regex::new(VERSION_REGEX) {
					Ok(regex) => {
						// Get version from output
						let version = match regex.captures(output_string.as_str()) {
							Some(captures) => captures.get(1).unwrap().as_str(),
							None => {
								return Err(String::from(format!(
									"Failed to parse from unionfs-fuse version output: \"{}\"",
									output_string.clone()
								)));
							}
						};

						// Return version
						version
					}
					Err(_) => {
						return Err(String::from(format!(
							"Failed to run regex in order to get unionfs-fuse version: \"{}\"",
							output_string
						)));
					}
				};

				// Check if version is valid
				if version.len() <= 0 {
					return Err(String::from(format!(
						"Failed to parse version from unionfs-fuse version output: \"{}\"",
						output_string
					)));
				}
			}
			Err(_) => {
				return Err(String::from(format!(
					"Failed to run unionfs-fuse version check: \"{}\"",
					command
				)));
			}
		}

		// Folder "dance" needed for Union FS - Fuse
		// 1) Change the name of the original mod folder (Ex. "Data") to UNION_FS_TEMP_FOLDER
		// 2) Create a new folder with the original name of the folder we just moved
		// 3) Use the old folder as a layer, and the new folder as the mount point

		// Replace last item in path with UNION_FS_TEMP_FOLDER
		let mount_temp = self.get_temp_mount_folder();

		// Should we move the mount folder?
		// Sometimes a previous deployment failed (is dirty) and the already moved mount folder is still there
		let mut should_move_folder = true;

		// Check if the target mount is dirty (we failed to unmount it previously)
		if mount_temp.exists() {
			let is_mount_temp_empty = match file_controller::is_folder_empty(&mount_temp) {
				Ok(result) => result,
				Err(e) => {
					println!("Error checking if mount temp folder is empty: {:?}", e);
					return Err(String::from(format!(
						"Error checking if mount temp folder is empty: {:?}",
						e
					)));
				}
			};

			if !is_mount_temp_empty {
				println!("Mount temp folder exists and has data!");
				should_move_folder = false;
			}
		}

		// Move the mount folder if needed
		if self.config.should_overlay_target && should_move_folder {
			println!(
				"Moving mount folder {:?} to {:?}",
				self.config.paths.target, mount_temp
			);

			// Move the mount folder to the temporal location
			match self.move_mount_folder() {
				Ok(_) => {}
				Err(e) => {
					println!("Error moving mount folder: {:?}", e);
					return Err(String::from(format!("Error moving mount folder: {:?}", e)));
				}
			}
		}

		// Parsed paths/layers for UnionFS-Fuse
		let mut parsed_layers: String = String::from("");

		fn path_to_str(path: &PathBuf) -> String {
			// path.to_str().unwrap().replace(" ", "\\ ")
			path.to_str().unwrap().to_string()
		}

		// Top layer is Overwrite folder, in RW mode
		parsed_layers
			.push_str(format!("{}=RW", path_to_str(&self.config.paths.overwrite)).as_str());

		// Invert lower layers order
		let inverted_sources: Vec<PathBuf> = self
			.config
			.paths
			.sources
			.clone()
			.into_iter()
			.rev()
			.collect();

		// Lower layers are the mods folders, in RO mode
		for path in inverted_sources {
			if !path.exists() {
				println!("Path {:?} does not exist", path);
				continue;
			}
			parsed_layers.push_str(format!(":{}=RO", path_to_str(&path)).as_str());
		}

		// Push the mount (temp) folder as the last layer, in RO mode
		// Only if we are in overlay mode
		if self.config.should_overlay_target {
			parsed_layers.push_str(format!(":{}=RO", path_to_str(&mount_temp)).as_str());
		}

		println!("Mounting UnionFS-Fuse with the following parameters:");
		println!("MountDir -  {:?}", self.config.paths.target);
		println!("MountDir - exists: {:?}", self.config.paths.target.exists());
		println!("TempMountDir - path: {:?}", mount_temp);
		println!("TempMountDir - exists: {:?}", mount_temp.exists());
		println!("WorkDir - path: {:?}", self.config.paths.workdir);
		println!("WorkDir - exists: {:?}", self.config.paths.workdir.exists());
		println!("Layers: {:?}", parsed_layers);

		let result = Command::new(command)
			.arg("-o")
			.arg("cow")
			.arg("-o")
			.arg("max_files=327680")
			.arg("-o")
			.arg("hide_meta_files")
			.arg("-o")
			.arg(format!("dirs={}", parsed_layers))
			.arg(path_to_str(&self.config.paths.target))
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
		let command_output = Command::new("umount")
			.arg(self.config.paths.target.clone())
			.output()
			.expect("failed to execute umount process");

		// Check if error
		if !command_output.status.success() {
			// Check if error is empty
			if command_output.stderr.is_empty() {
				return Err(format!(
					"Failed to unmount with status: {:?}",
					command_output.status.code()
				));
			}

			// If the error is that the folder is not mounted, return success
			if String::from_utf8(command_output.stderr.clone())
				.expect("Failed to parse umount error output")
				.contains("not mounted")
			{
				return Ok(());
			}

			// Return error
			return Err(format!("Failed to unmount: {:?}", command_output.stderr));
		}

		// Wait for a bit, so that the mount folder is not busy
		thread::sleep(Duration::from_millis(100));

		// Restore mount if we are in overlay mode
		if self.config.should_overlay_target {
			self.restore_mount_folder()?;
		}

		return Ok(());
	}
}
