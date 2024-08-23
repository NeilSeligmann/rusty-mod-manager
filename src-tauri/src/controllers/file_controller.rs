use compress_tools::{uncompress_archive, Ownership};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::vec;
use unrar::Archive;

pub fn save_file(path: PathBuf, data: &[u8]) -> std::io::Result<()> {
	let prefix = path.parent().unwrap();
	std::fs::create_dir_all(path.parent().unwrap())?;

	// If file exists, move it to backup
	// if path.exists() {
	// 	let backup_path = prefix.join(format!(
	// 		"{}.backup",
	// 		path.file_name().unwrap().to_str().unwrap()
	// 	));
	// 	std::fs::rename(&path, &backup_path)?;
	// }

	let mut file = File::create(path)?;
	file.write_all(data)?;

	Ok(())
}

pub fn save_file_with_backup(path: PathBuf, data: &[u8]) -> std::io::Result<()> {
	// If file exists, move it to backup
	if path.exists() {
		// println!("File {} already exists, moving as backup", path.display());
		let prefix = path.parent().unwrap();
		let backup_path = prefix.join(format!(
			"{}.backup",
			path.file_name().unwrap().to_str().unwrap()
		));
		std::fs::rename(&path, &backup_path)?;
		// println!("Moved {} to {}", path.display(), backup_path.display());
	}

	return save_file(path, data);
}

// pub fn save_file_rolling(path: PathBuf, data: &[u8], roll_count: usize) -> std::io::Result<()> {
// 	// If file exists, roll it
// 	if path.exists() {
// 		roll_file(path.clone(), roll_count, 0)?;
// 	}

// 	// Save the file
// 	let mut file = File::create(path)?;
// 	file.write_all(data)?;

// 	Ok(())
// }

// fn roll_file(path: PathBuf, roll_count: usize, current_index: usize) -> std::io::Result<()> {
// 	let parent_path = path.parent().unwrap();
// 	let file_name = path.file_name().unwrap().to_str().unwrap();
// 	let current_index_path = match current_index == 0 {
// 		true => parent_path.join(file_name),
// 		false => parent_path.join(format!("{}.{}", file_name, current_index)),
// 	};

// 	// If current file exists, roll it (recursive), return ok
// 	if current_index_path.exists() {
// 		if !path.is_file() {
// 			return Err(std::io::Error::new(
// 				std::io::ErrorKind::Other,
// 				format!("{} is not a file, cannot save-roll it.", path.display()),
// 			));
// 		}

// 		let new_index = current_index + 1;

// 		// If we are moving the file, but the index is higher than the roll count
// 		// delete the file
// 		if new_index >= roll_count {
// 			delete_file_if_exists(path.clone())?;
// 		} else {
// 			// Here we need to move the file
// 			// New Path = (path) + "." + (current_index)
// 			let new_path = path.parent().unwrap().join(format!(
// 				"{}.{}",
// 				path.file_name().unwrap().to_str().unwrap(),
// 				current_index + 1
// 			));

// 			// If the new path already exists, roll it recursively
// 			if new_path.exists() {
// 				roll_file(path, roll_count, new_index)?;
// 			} else {
// 				// Move the file, from current index path to new path
// 				std::fs::rename(current_index_path, new_path)?;
// 			}
// 		}
// 	}
// 	// The current index file does not exist, return ok

// 	return Ok(());
// }

pub fn read_file(path: PathBuf) -> std::io::Result<String> {
	let mut file = File::open(path)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	Ok(contents)
}

pub fn read_file_bytes(path: PathBuf) -> std::io::Result<Vec<u8>> {
	let mut file = File::open(path)?;
	let mut contents = Vec::new();
	file.read_to_end(&mut contents)?;
	Ok(contents)
}

pub fn list_files_recursively_flattened(path: PathBuf) -> std::io::Result<Vec<String>> {
	let mut files = Vec::new();

	for entry in std::fs::read_dir(path)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			files.append(&mut list_files_recursively_flattened(path)?);
		} else {
			files.push(path.to_str().unwrap().to_string());
		}
	}

	Ok(files)
}

pub fn list_files_recursively_relative_flattened(path: PathBuf) -> std::io::Result<Vec<String>> {
	let mut flattened_files = list_files_recursively_flattened(path.clone())?;

	// Remove the root path
	for file in &mut flattened_files {
		*file = file.replace(path.clone().to_str().unwrap(), "");
	}

	Ok(flattened_files)
}

pub fn list_entries_absolute_path(path: PathBuf) -> std::io::Result<Vec<PathBuf>> {
	let mut entries: Vec<PathBuf> = Vec::new();
	for entry in std::fs::read_dir(path)? {
		let entry = entry?;
		let path = entry.path();

		entries.push(path);
	}

	return Ok(entries);
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct FileStructureSegment {
	segment: String,
	isFile: bool,
	children: Option<Vec<FileStructureSegment>>,
}

impl FileStructureSegment {
	fn get_segment(&self) -> String {
		// if self.segment == "/" {
		// 	return String::from("");
		// }
		self.segment.clone()
	}

	pub fn case_fold_path(&self, vec_path: Vec<String>) -> Option<String> {
		let mut path = vec_path.clone();

		// If we are the root, insert to the beginning
		// if self.segment == "/" {
		// 	if self.children.is_some() {
		// 		for child in self.children.as_ref().unwrap() {
		// 			if let Some(path) = child.case_fold_path(path.clone()) {
		// 				return Some(format!("{}/{}", self.get_segment(), path));
		// 			}
		// 		}
		// 	}

		// 	return Some(vec_path.join("/").to_lowercase());
		// }

		// Remove and get first item, cast to lowercase
		let first_item = path.remove(0).to_lowercase();

		// If the first segment in vec_path is the current segment, continue
		// If not, return none

		// If the first segment is not the current segment, return the given string in lowercase
		// The given path does not exist, therefore we case-fold the given path and return it as is
		if first_item != self.get_segment().to_lowercase() {
			return None;
			// return Some(vec_path.join("/").to_lowercase());
		}

		// If there are no more segments, return the current segment
		if path.is_empty() {
			return Some(self.get_segment());
		}

		// Now we need to check each child segment to see if the match continues
		if self.children.is_some() {
			for child in self.children.as_ref().unwrap() {
				if let Some(path) = child.case_fold_path(path.clone()) {
					return Some(format!("{}/{}", self.get_segment(), path));
				}
			}
		}

		// If no child matches, we return the current segment with the rest case-folded
		return Some(format!(
			"{}/{}",
			self.get_segment(),
			path.join("/").to_lowercase()
		));
	}
}

pub fn list_file_structure_relatively(
	base_path: PathBuf,
) -> std::io::Result<Vec<FileStructureSegment>> {
	// println!("List file structure recursively: {:?}", base_path);
	let mut entries: Vec<FileStructureSegment> = Vec::new();

	for entry in std::fs::read_dir(base_path)? {
		// println!("List file entry: {:?}", entry);
		let entry = entry?;
		let entry_path = entry.path();

		let file_name = entry_path.file_name();
		if file_name.is_none() {
			continue;
		}
		let file_name_string = file_name.unwrap().to_str().unwrap().to_string();

		if entry_path.is_dir() {
			// let children = list_file_structure_relatively(entry_path.join(file_name_string.clone()));
			let children = list_file_structure_relatively(entry_path);

			entries.push(FileStructureSegment {
				segment: file_name_string,
				isFile: false,
				children: match children {
					Ok(children) => Some(children),
					Err(_) => None,
				},
			})
		} else {
			// println!("Adding file to structure: {}", file_name_string);
			entries.push(FileStructureSegment {
				segment: file_name_string,
				isFile: true,
				children: None,
			})
		}
	}

	return Ok(entries);
}

pub fn file_exists(path: &PathBuf) -> bool {
	path.try_exists().unwrap_or(false)
}

pub fn copy_recursive(from: PathBuf, to: PathBuf, hardlink: bool) -> std::io::Result<()> {
	println!("Copying recursive {} to {}", from.display(), to.display());

	let mut command = std::process::Command::new("cp");
	command
		.arg(match hardlink {
			true => "-arlf",
			false => "-arf",
		})
		.arg("-T")
		.arg(from)
		.arg(to);

	command.output()?;
	Ok(())
}

// This function deletes a folder, implementing safe checks
// CONS: Slow, inefficient, safe checks are simple
pub fn delete_folder_safe(path: PathBuf, parent_path: PathBuf) -> std::io::Result<()> {
	println!("Deleting folder: {}", path.display());
	println!("Parent path: {}", parent_path.display());

	let denied_paths = Vec::from(["/", "/home", "/root", "/usr", "/var", "/dev"]);

	// for check_path in denied_paths {
	// 	if parent_path.to_str().unwrap().starts_with(check_path) {
	// 		return Err(std::io::Error::new(
	// 			std::io::ErrorKind::Other,
	// 			"Parent path is in deny list",
	// 		));
	// 	}
	// }

	// Check parent_path is not in check_paths
	if denied_paths.contains(&parent_path.to_str().unwrap()) {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Parent path is denied",
		));
	}

	// Check "path" is child of "parent_path"
	if !path.starts_with(parent_path) {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"Path is not a child of parent_path",
		));
	}

	return std::fs::remove_dir_all(path);
}

pub fn delete_folder_if_empty(path: PathBuf) -> std::io::Result<()> {
	std::fs::remove_dir(path)?;

	Ok(())
}

pub fn create_folder(path: &PathBuf) -> std::io::Result<()> {
	std::fs::create_dir_all(path)?;

	Ok(())
}

pub fn move_folder(source: PathBuf, destination: PathBuf) -> Result<(), String> {
	// if !source.is_dir() {
	// 	return Err("Source is not a directory".to_string());
	// }

	let move_file = move_file(source.clone(), destination.clone())
		.map_err(|e| format!("Failed to move folder: {}", e.to_string()));

	match move_file {
		Ok(_) => {
			return Ok(());
		}
		Err(err) => {
			if !err.to_lowercase().contains("directory not empty") {
				return Err(err);
			}
		}
	}

	// We could not move the folder due to the target not being empty, therefore we move its contents
	let source_str = source.to_str().unwrap();
	let destination_str = destination.to_str().unwrap();
	let command = format!("mv \"{}\"/* \"{}/\"", source_str, destination_str);

	// Create and wait the move process
	let output = std::process::Command::new("sh")
		.arg("-c")
		.arg(command)
		.output()
		.expect("Failed to move directory");

	if !output.status.success() {
		let err_output = std::str::from_utf8(&output.stderr).unwrap();
		println!(
			"Failed to move directory: {} -> {}",
			source.display(),
			destination.display()
		);
		println!("Failed to move directory: {}", err_output);
		return Err(format!("Failed to move directory: \"{}\"", err_output));
	}

	// Delete old empty directory
	delete_folder_if_empty(source)
		.map_err(|e| format!("Failed to delete delete empty folder: {}", e.to_string()))?;

	return Ok(());
}

// Case fold entire folder (recursively) will not affect root folder
pub fn case_fold_folder_recursive(path: PathBuf) -> std::io::Result<()> {
	// println!("Case folding folder: {}", path.display());

	for entry in std::fs::read_dir(path)? {
		let entry = entry?;
		let entry_path = entry.path();
		// let relative_path = entry_path.strip_prefix(path.clone())?;

		// We move the file/folder to the same location, but with lowercase name
		let new_path = entry_path.parent().unwrap().join(
			entry_path
				.clone()
				.file_name()
				.unwrap()
				.to_str()
				.unwrap()
				.to_lowercase(),
		);

		// Move the file/folder
		// println!("Moving {} to {}", entry_path.display(), new_path.display());
		match move_file(entry_path.clone(), new_path.clone()) {
			Ok(_) => (),
			Err(err) => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::Other,
					format!(
						"[Case-fold] Failed to move file/folder: \"{}\" -> \"{}\"",
						entry_path.display(),
						err
					),
				))
			}
		}

		// if entry_path.is_file() {
		// 	// move_file(entry_path.clone(), new_path.clone())?;
		// } else if new_path.is_dir() {
		if new_path.is_dir() {
			// match move_folder(entry_path.clone(), new_path.clone()) {
			// 	Ok(_) => (),
			// 	Err(err) => {
			// 		return Err(std::io::Error::new(
			// 			std::io::ErrorKind::Other,
			// 			// "Failed to move folder",
			// 			format!("Failed to move file/folder: {}", err),
			// 		))
			// 	}
			// }
			case_fold_folder_recursive(new_path)?;
		}
	}

	return Ok(());
}

pub fn move_file(source: PathBuf, destination: PathBuf) -> std::io::Result<()> {
	// Create the destination folder if it doesn't exist
	std::fs::create_dir_all(destination.parent().unwrap())?;

	return std::fs::rename(source, destination);
}

pub fn get_files_in_folder_with_extensions(path: PathBuf, extensions: Vec<&str>) -> Vec<PathBuf> {
	let mut files: Vec<PathBuf> = Vec::new();

	// println!(
	// 	"Getting files ({}) in folder: {}",
	// 	extensions.join(", "),
	// 	path.display()
	// );

	for entry in std::fs::read_dir(path).unwrap() {
		let entry = entry.unwrap();
		let entry_path = entry.path();
		if entry_path.is_file()
			&& extensions.contains(&entry_path.extension().unwrap().to_str().unwrap())
		{
			files.push(entry_path);
		}
	}

	// println!("Found {} files", files.len());

	return files;
}

pub fn is_folder_empty(target: &PathBuf) -> Result<bool, String> {
	let entries = match std::fs::read_dir(target) {
		Ok(entries) => entries,
		Err(err) => return Err(format!("Failed to read directory: {}", err)),
	};

	return Ok(entries.count() == 0);
}

pub fn open_folder(path: PathBuf) -> Result<(), String> {
	let path_str = path.to_str().unwrap();
	let command = format!("xdg-open \"{}\"", path_str);

	return match std::process::Command::new("sh")
		.arg("-c")
		.arg(command)
		.spawn()
	{
		Ok(_) => Ok(()),
		Err(_) => Err("Failed to open folder".to_string()),
	};
}

pub fn delete_file_if_exists(path: PathBuf) -> std::io::Result<()> {
	if path.exists() {
		std::fs::remove_file(path)?;
	}

	Ok(())
}

pub fn open_in_filemanager(path: PathBuf) -> Result<(), String> {
	let path_str = path.to_str().unwrap();
	// let command = format!("dbus-send --session --dest=org.freedesktop.FileManager1 --type=method_call /org/freedesktop/FileManager1 org.freedesktop.FileManager1.ShowItems array:string:\"{}\"", path_str);
	let command = format!("dbus-send --session --print-reply --dest=org.freedesktop.FileManager1 --type=method_call /org/freedesktop/FileManager1 org.freedesktop.FileManager1.ShowItems array:string:\"file://{}\" string:\"\"", path_str);

	return match std::process::Command::new("sh")
		.arg("-c")
		.arg(command)
		.spawn()
	{
		Ok(_) => Ok(()),
		Err(_) => Err("Failed to open filemanager".to_string()),
	};
}

pub fn extract_archive(source: PathBuf, destination: PathBuf) -> Result<(), String> {
	println!(
		"Extracting archive: {} -> {}",
		source.display(),
		destination.display()
	);

	// Create destination folder
	create_folder(&destination).map_err(|err| {
		format!(
			"Failed to create extraction destination folder: {}",
			err.to_string()
		)
	})?;

	let extension = source.extension();
	println!("Extension: {:?}", extension);

	if extension.is_some() {
		println!("Extension is some!");
		if extension.unwrap().to_str().unwrap() == "rar" {
			println!("IS RAR!");
			println!("Extracting rar archive: {}", source.display());
			extract_rar_archive(source, destination)
				.map_err(|err| format!("Failed to extract rar archive: {}", err.to_string()))?;

			return Ok(());
		}
	}

	// Read archive
	let readFile = File::open(source)
		.map_err(|err| format!("Failed to read file while extracting: {}", err.to_string()))?;

	// Uncompress archive
	uncompress_archive(readFile, destination.as_path(), Ownership::Ignore)
		.map_err(|err| format!("Failed to uncompress archive: {}", err.to_string()))?;

	Ok(())
}

fn extract_rar_archive(
	source: PathBuf,
	destination: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
	let mut archive = Archive::new(&source.clone())
		.open_for_processing()
		.expect("Failed to open rar archive for processing");

	while let Some(header) = archive.read_header()? {
		println!(
			"{} bytes: {}",
			header.entry().unpacked_size,
			header.entry().filename.to_string_lossy(),
		);

		archive = if header.entry().is_file() {
			// header.extract()?;
			header.extract_with_base(&destination)?
		} else {
			header.skip()?
		};
	}

	Ok(())
}

pub fn join_paths(path1: PathBuf, path2: PathBuf) -> PathBuf {
	let stripped_path2 = path2.strip_prefix("/").unwrap_or(&path2);
	return path1.join(stripped_path2);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn file_structure_segment_case_folding_works() {
		let file_structure = FileStructureSegment {
			segment: String::from("Segment1"),
			isFile: false,
			children: Some(vec![
				FileStructureSegment {
					segment: String::from("Segment2"),
					isFile: false,
					children: None,
				},
				FileStructureSegment {
					segment: String::from("Segment3"),
					isFile: false,
					children: None,
				},
				FileStructureSegment {
					segment: String::from("Segment4-File"),
					isFile: true,
					children: None,
				},
			]),
		};

		// If the complete path exists, it should return in matching the casing
		assert_eq!(
			file_structure.case_fold_path(vec!["Segment1".to_string()]),
			Some(String::from("Segment1"))
		);
		assert_eq!(
			file_structure.case_fold_path(vec!["Segment1".to_string(), "Segment2".to_string()]),
			Some(String::from("Segment1/Segment2"))
		);

		// If the beginning of the path exists, it should return only that part matching the casing
		assert_eq!(
			file_structure.case_fold_path(vec![
				"Segment1".to_string(),
				"Some-Non-Existing-Path".to_string()
			]),
			Some(String::from("Segment1/some-non-existing-path"))
		);
		assert_eq!(
			file_structure.case_fold_path(vec![
				"Segment1".to_string(),
				"Segment2".to_string(),
				"Some-Non-Existing-Path".to_string()
			]),
			Some(String::from("Segment1/Segment2/some-non-existing-path"))
		);
		assert_eq!(
			file_structure.case_fold_path(vec![
				"Segment1".to_string(),
				"Segment2".to_string(),
				"AFile.ext".to_string()
			]),
			Some(String::from("Segment1/Segment2/afile.ext"))
		);

		// If the path is not in the structure, it should return None
		assert_eq!(
			file_structure.case_fold_path(vec!["Path-that-is-not-in-the-structure".to_string()]),
			None
		);
		assert_eq!(
			file_structure
				.case_fold_path(vec!["Inexistent-path1".to_string(), "Path2".to_string()]),
			None
		);
	}
}
