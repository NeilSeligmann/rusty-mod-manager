# Rusty Mod Manager
A Linux-Native Mod Manager, built to be fast and simple to use.

Made using [Rust](https://www.rust-lang.org/) and [Tauri](https://tauri.app/).

> [!WARNING]
> This project is in active development!
> 
> Features and functionaliy might change. Updates will most probably contain breaking changes.


![Mods Window](/readme/Window-Mods.png)


## Features
- Made for Linux
- Fast 🚀
- Lightweight 🪶
- FOMOD Support (WIP)
- Mod Versioning
- Virtual File-System Deployment
  - Uses [UnionFS-Fuse](https://github.com/rpodgorny/unionfs-fuse)
  - Saves Folder Virtualization
  - Configuration Folder Virtualization
- Download Manager
  - Multi-threaded
  - Resumable Downloads
  - Nexusmods / NXM Scheme Support
- WINE / Proton Compatibility
  - Case-Folding

## Planned Features & Improvements
- Plugin Support (WIP) _(Bethesda games only)_
- Profiles
- BSA Extraction
- Improved error handling
  - Currently most errors are handled as strings, not good.
- Planned games support:
  - Cyperpunk 2077
- Migrate to Tuari v2
- Multiple instances open in simultaneous
- Run executable directly by specifying an argument
- Steam Games Auto-detection
- Automatic load-order using LOOT
- Add tests

## Supported Games
Currently, the mod manager is being developed with Bethesda games (Skyrim specially) in mind. There are plans for supporting more games in the future!

Rusty Mod Manager manages your mods and downloads, but in the end these are simply folders.
While the games you want to play may not be specifically supported, they might still work.

## How does it work?
When running a game/executable the mod manager deploys a Virtual File-System (VFS), using [UnionFS-Fuse](https://github.com/rpodgorny/unionfs-fuse). This makes all of your enabled mods appear transaprently to the game you are running.

## Requirements
- [UnionFS-Fuse](https://github.com/rpodgorny/unionfs-fuse): For deploying the mods/saves/settings
- icoutils/wrestool _(Optional)_: Used for extracting `.exe` file data

## Handle NXM Scheme
In order to handle NXM links you need to run the executable with the argument `./RustyModManager nxm [NXM-LINK-HERE]`.

You can create a `.desktop` file in your applications (Ex. `~/.local/share/applications`) with the content:
```ini
[Desktop Entry]
Categories=Game;
Name=Rusty Mod Manager (NXM Handler)
Comment=A native mod manager for linux.
Exec={PATH_TO_EXECUTABLE} nxm %u
Path={PATH_TO_EXECUTABLE_FOLDER}
MimeType=x-scheme-handler/nxm;
Terminal=false
Type=Application
```

Replace `{PATH_TO_EXECUTABLE}` and `{PATH_TO_EXECUTABLE_FOLDER}` with their respective values.

For now it is recommended that you have an instance of the mod manager already open, so it will receive the IPC request to download the file.

## Development
The mod manager has been developed using [Rust](https://www.rust-lang.org/) + [Tauri](https://tauri.app/) + [Bun](https://bun.sh/).

How to run locally:
1. Clone this repo
2. Install dependencies with `bun install`
3. Run project with `bun run dev`

## Disclaimer
This software is provided as-is, without warranty.

The code is licensed under [GNU GPL v3](./LICENSE)
