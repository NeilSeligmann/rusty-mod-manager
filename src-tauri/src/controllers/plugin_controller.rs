use std::path::{Path, PathBuf};

use crate::instances::GameIdentifier;
use esplugin::{ParseOptions, Plugin};
use loadorder::{GameSettings, WritableLoadOrder};

impl From<GameIdentifier> for loadorder::GameId {
	fn from(game_identifier: GameIdentifier) -> Self {
		match game_identifier {
			GameIdentifier::Generic => {
				panic!("Cannot cast Generic game identifier to LoadOrder GameId")
			}
			GameIdentifier::Oblivion => loadorder::GameId::Oblivion,
			GameIdentifier::Morrowind => loadorder::GameId::Morrowind,
			GameIdentifier::Skyrim => loadorder::GameId::Skyrim,
			GameIdentifier::SkyrimSE => loadorder::GameId::SkyrimSE,
			GameIdentifier::Fallout3 => loadorder::GameId::Fallout3,
			GameIdentifier::FalloutNV => loadorder::GameId::FalloutNV,
			GameIdentifier::Fallout4 => loadorder::GameId::Fallout4,
		}
	}
}

impl From<GameIdentifier> for esplugin::GameId {
	fn from(game_identifier: GameIdentifier) -> Self {
		match game_identifier {
			GameIdentifier::Generic => {
				panic!("Cannot cast Generic game identifier to LoadOrder GameId")
			}
			GameIdentifier::Oblivion => esplugin::GameId::Oblivion,
			GameIdentifier::Morrowind => esplugin::GameId::Morrowind,
			GameIdentifier::Skyrim => esplugin::GameId::Skyrim,
			GameIdentifier::SkyrimSE => esplugin::GameId::SkyrimSE,
			GameIdentifier::Fallout3 => esplugin::GameId::Fallout3,
			GameIdentifier::FalloutNV => esplugin::GameId::FalloutNV,
			GameIdentifier::Fallout4 => esplugin::GameId::Fallout4,
		}
	}
}

// pub fn cast_game_identifier_to_game_id(game_identifier: GameIdentifier) -> GameId {
// 	let identifier = match game_identifier {
// 		GameIdentifier::Generic => {
// 			panic!("Cannot cast Generic game identifier to Plugin GameId")
// 		}
// 		GameIdentifier::Oblivion => GameId::Oblivion,
// 		GameIdentifier::Morrowind => GameId::Morrowind,
// 		GameIdentifier::Skyrim => GameId::Skyrim,
// 		GameIdentifier::SkyrimSE => GameId::SkyrimSE,
// 		GameIdentifier::Fallout3 => GameId::Fallout3,
// 		GameIdentifier::FalloutNV => GameId::FalloutNV,
// 		GameIdentifier::Fallout4 => GameId::Fallout4,
// 	};

// 	return identifier;
// }

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct BethesdaPlugin {
	description: Option<String>,
	#[specta(type = String)]
	override_record_count: usize,
	file_name: Option<String>,
	header_version: Option<f32>,
	is_light_plugin: bool,
	is_master_file: bool,
	is_valid_as_light_plugin: bool,
	masters: Vec<String>,
	record_and_group_count: Option<u32>,
	overlaps_with: Option<Vec<String>>,
}

pub fn read_plugin(
	game_identifier: GameIdentifier,
	file_path: &Path,
) -> Result<BethesdaPlugin, String> {
	println!("Reading plugin at \"{}\"", file_path.to_str().unwrap());

	let mut plugin = Plugin::new(esplugin::GameId::from(game_identifier), file_path);

	// We need to actually parse the file
	match plugin.parse_file(ParseOptions::whole_plugin()) {
		Ok(_) => {}
		Err(err) => {
			return Err(format!(
				"Error parsing plugin at \"{}\": {}",
				file_path.to_str().unwrap(),
				err
			))
		}
	}

	let description = match plugin.description() {
		Ok(description) => description,
		Err(err) => return Err(format!("Error reading plugin description: {}", err)),
	};

	let masters = match plugin.masters() {
		Ok(masters) => masters,
		Err(err) => return Err(format!("Error reading plugin masters: {}", err)),
	};

	let override_record_count = match plugin.count_override_records() {
		Ok(count) => count,
		Err(err) => {
			return Err(format!(
				"Error reading plugin override record count: {}",
				err
			))
		}
	};

	let is_valid_as_light_plugin = match plugin.is_valid_as_light_plugin() {
		Ok(is_valid_as_light_plugin) => is_valid_as_light_plugin,
		Err(err) => {
			return Err(format!(
				"Error reading plugin is_valid_as_light_plugin: {}",
				err
			))
		}
	};

	let parsed_plugin = BethesdaPlugin {
		description,
		override_record_count,
		file_name: plugin.filename(),
		header_version: plugin.header_version(),
		is_light_plugin: plugin.is_light_plugin(),
		is_master_file: plugin.is_master_file(),
		is_valid_as_light_plugin,
		masters,
		record_and_group_count: plugin.record_and_group_count(),
		overlaps_with: None,
	};

	return Ok(parsed_plugin);
}

pub fn read_load_order(
	game_identifier: GameIdentifier,
	game_path: &Path,
	local_path: &Path,
	my_games_path: PathBuf,
) -> Result<GameSettings, String> {
	// println!("Reading load order at \"{}\"", game_path.to_str().unwrap());

	// /mnt/980pro2tb/SteamLibrary/steamapps/compatdata/489830/pfx/drive_c/users/steamuser/AppData/Local/Skyrim Special Edition/

	let game_settings = match GameSettings::with_local_and_my_games_paths(
		loadorder::GameId::from(game_identifier),
		game_path,
		local_path,
		my_games_path,
	) {
		Ok(settings) => settings,
		Err(err) => return Err(err.to_string()),
	};

	// game_settings.set_additional_plugins_directories(paths)

	// let game_settings = match GameSettings::(
	// 	loadorder::GameId::from(game_identifier),
	// 	game_path,
	// 	local_path,
	// ) {
	// 	Ok(settings) => settings,
	// 	Err(err) => return Err(err.to_string()),
	// };

	// let game_settings = match GameSettings::new(loadorder::GameId::from(game_identifier), game_path)
	// {
	// 	Ok(settings) => settings,
	// 	Err(err) => return Err(err.to_string()),
	// };

	return Ok(game_settings);
}
