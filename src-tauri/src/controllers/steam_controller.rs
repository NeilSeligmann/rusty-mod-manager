extern crate steamlocate;
use std::{collections::HashMap, path::PathBuf};

use steamlocate::{SteamApp, SteamDir};

use crate::instances::{GameIdentifier, InstanceExecutable};

#[taurpc::ipc_type]
pub struct SupportedSteamGamePaths {
	mods_root: PathBuf,
	profiles_root: PathBuf,
	saves_root: PathBuf,
}

#[taurpc::ipc_type]
pub struct SupportedSteamGame {
	app_id: u32,
	public_name: String,
	known_binaries: Vec<InstanceExecutable>,
	paths: SupportedSteamGamePaths,
	game_identifier: Option<GameIdentifier>,
}

#[taurpc::ipc_type]
pub struct FoundSteamGame {
	absolute_path: String,
	steam_game: SupportedSteamGame,
}

fn find_steam_apps() -> Option<HashMap<u32, Option<SteamApp>>> {
	let mut dir: SteamDir = SteamDir::locate()?;
	let apps = dir.apps().clone();

	// Clean apps, if value is none remove it
	// let cleaned_apps: HashMap<u32, SteamApp> =
	// 	apps.into_iter().filter(|(_, v)| v.is_some()).collect();

	return Some(apps);
}

pub fn scan_for_steam_games(supported_games: &Vec<SupportedSteamGame>) -> Vec<FoundSteamGame> {
	let mut available_games: Vec<FoundSteamGame> = Vec::new();

	let steam_apps = find_steam_apps().unwrap_or(HashMap::new());

	for supported_game in supported_games {
		let steam_app_option = steam_apps.get(&supported_game.app_id);

		if !steam_app_option.is_some() {
			continue;
		}

		let steam_app = steam_app_option.unwrap();
		if steam_app.is_none() {
			continue;
		}

		let found_steam_game = FoundSteamGame {
			absolute_path: steam_app
				.clone()
				.unwrap()
				.path
				.to_str()
				.unwrap()
				.to_string(),
			steam_game: supported_game.clone(),
		};

		available_games.push(found_steam_game);
	}

	return available_games;
}
