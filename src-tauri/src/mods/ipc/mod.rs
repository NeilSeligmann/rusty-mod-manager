use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::{fmt, thread};
// use tauri::api::{cli, ipc};
use tauri::{AppHandle, Manager};

use crate::controllers::file_controller;

// #[derive(Clone, Deserialize, Serialize)]
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct IPCPayload {
	// raw_message: String,
	pub command: String,
	pub args: Vec<String>,
}

impl fmt::Display for IPCPayload {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} -{}", self.command, self.args.join(" -"))
	}
}

fn socket_path() -> PathBuf {
	return PathBuf::from("/tmp/rusty-mod-manager.sock");
}

// -------
// Client
// -------

pub struct IPCClient {
	stream_client: Mutex<Option<UnixStream>>,
}

impl IPCClient {
	pub fn new() -> Self {
		Self {
			stream_client: Mutex::new(None),
		}
	}

	pub fn connect_to_socket(&mut self) -> std::io::Result<()> {
		let mut stream_client = self.stream_client.lock().unwrap();

		if stream_client.is_some() {
			return Ok(());
		}

		let client = UnixStream::connect(socket_path())?;
		*stream_client = Some(client);

		return Ok(());
	}

	pub fn send_payload_to_stream(&mut self, payload: IPCPayload) -> std::io::Result<String> {
		match self.connect_to_socket() {
			Ok(_) => {}
			Err(e) => {
				return Err(e);
			}
		}

		let mut stream_client_opt = self.stream_client.lock().unwrap();
		let stream_client = stream_client_opt.as_mut().unwrap();

		stream_client.set_read_timeout(Some(std::time::Duration::from_millis(5000)))?;

		let deserialized = serde_json::to_string(&payload)?;
		stream_client.write_all(deserialized.as_bytes())?;

		println!("[IPC] Sent message to stream: \"{}\"", deserialized);

		let mut response = [0 as u8; 2048];
		stream_client.read(&mut response)?;

		let string_response = String::from_utf8_lossy(&response);

		println!(
			"[IPC] Received response from stream: \"{}\"",
			string_response.to_string()
		);

		return Ok(string_response.to_string());
	}

	pub fn socket_path_exists(&self) -> bool {
		return socket_path().exists();
	}

	pub fn ping_socket(&mut self) -> std::io::Result<()> {
		let response = self.send_payload_to_stream(IPCPayload {
			command: "ping".to_string(),
			args: Vec::new(),
		})?;

		if !response.contains("pong") {
			return Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				"Invalid ping response from socket".to_string(),
			));
		}

		return Ok(());
	}
}

// -------
// Server
// -------

pub struct IPCServer {
	pub app_handle: AppHandle,
}

impl IPCServer {
	pub fn initialize_listener(&self, initial_ipc_payload: Option<IPCPayload>) {
		// Check if socket exists
		// let exists = socket_path().exists();

		if socket_path().exists() {
			println!("[IPC] Socket file exists, assumming it is stale and deleting it.");
			match file_controller::delete_file_if_exists(socket_path()) {
				Ok(_) => {}
				Err(e) => {
					println!("[IPC] Failed to delete socket file: {}", e.to_string());
					return;
				}
			}
		}

		// Create socket
		let listener = match UnixListener::bind(socket_path()) {
			Ok(listener) => listener,
			Err(e) => {
				println!("[IPC] Failed to create socket: {}", e.to_string());
				return;
			}
		};

		// Handle IPC message if there is one
		if initial_ipc_payload.is_some() {
			let handle_clone = self.app_handle.clone();
			let initial_ipc_payload = initial_ipc_payload.unwrap();
			thread::spawn(move || {
				// Wait for a few seconds before handling the initial IPC message
				thread::sleep(Duration::from_millis(3000));

				// Handle the initial IPC message
				handle_ipc_message(initial_ipc_payload, handle_clone);
			});
		}

		// Accept connections and process them, spawning a new thread for each one
		for stream in listener.incoming() {
			match stream {
				Ok(stream) => {
					let handle_clone = self.app_handle.clone();
					thread::spawn(move || handle_client(handle_clone, stream));
				}
				Err(err) => {
					// Connection failed
					println!("[IPC] Error handling socket client: {}", err);
					break;
				}
			}
		}

		// close the socket server
		drop(listener);
	}
}

fn handle_client(app_handle: AppHandle, stream: UnixStream) {
	let mut stream = stream;
	let mut data = [0 as u8; 2048];

	loop {
		match stream.read(&mut data) {
			Ok(size) => {
				if size == 0 {
					break;
				}

				let message = String::from_utf8_lossy(&data[0..size]).to_string();
				println!("[IPC] Received message: \"{}\"", message.replace("\n", ""));

				match serde_json::from_str::<IPCPayload>(&message) {
					Ok(payload) => {
						let response = handle_ipc_message(payload, app_handle.clone());
						match stream.write_all(response.as_bytes()) {
							Ok(_) => {
								println!("[IPC] Sent response: \"{}\"", response.replace("\n", ""));
							}
							Err(e) => {
								println!("[IPC] Failed to send response: {}", e.to_string());
								break;
							}
						}
					}
					Err(_) => {
						println!("[IPC][ERR] Invalid payload received: \"{}\"", message);

						match stream.write_all("invalid_payload".as_bytes()) {
							Ok(_) => {
								println!("[IPC][ERR] Sent invalid payload response.");
							}
							Err(e) => {
								println!(
									"[IPC][ERR] Failed to send invalid payload response: {}",
									e.to_string()
								);
								break;
							}
						}
					}
				};
			}
			Err(_) => {
				break;
			}
		}
	}
}

fn handle_ipc_message(ipc_payload: IPCPayload, app_handle: AppHandle) -> String {
	// let ipc_payload = match serde_json::from_str::<IPCPayload>(&payload) {
	// 	Ok(payload) => payload,
	// 	Err(_) => return "invalid_payload".to_string(),
	// };

	let response = match ipc_payload.command.as_str() {
		"ping" => "pong".to_string(),
		"nxm" => "ok".to_string(),
		_ => {
			return "unknown_command".to_string();
		}
	};

	let emit_event = app_handle.emit_all("ipc", ipc_payload.clone());
	match emit_event {
		Ok(_) => {
			println!("[IPC] IPC event emitted to front-end: {}", ipc_payload);
		}
		Err(e) => {
			println!(
				"[IPC] Error sending IPC message to front-end: {}",
				e.to_string()
			);
		}
	}

	return response;
}
