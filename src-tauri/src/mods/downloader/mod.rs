use events::{DownloaderEvent, DownloaderEventHandler};
use futures::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, RANGE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::controllers::file_controller;
use crate::mods::downloader::events::DownloaderChunkEventHandler;
use crate::state::ApplicationState;
use crate::ApiDownloadsEventTrigger;
use file_integrity::hash_file;
use std::time::{Duration, SystemTime};

use self::events::DownloaderChunkEvent;

pub mod events;

// Serde Defaults
pub fn default_date() -> String {
	return SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.to_string();
}

#[derive(Type, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum DownloadStatus {
	Queued,
	Downloading,
	Merging,
	Downloaded,
	Verifying,
	Failed,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct DownloadNexusData {
	pub mod_id: String,
	pub file_id: String,
}

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct Download {
	pub file_name: String,
	pub status: DownloadStatus,
	pub size_total: String,
	pub size_downloaded: String,
	pub url: String,
	pub md5: Option<String>,
	// pub is_initialized: bool,
	// We dont want to either send this to the front
	// or save it to the disk
	pub error: Option<String>,
	#[serde(skip)]
	pub downloader: Option<Downloader>,
	#[serde(skip)]
	pub pending_update: bool,
	// Dates
	#[serde(default = "default_date")]
	pub added_at: String,
	pub completed_at: Option<String>,
	// Nexus Data
	pub nexus_data: Option<DownloadNexusData>,
}

impl Download {
	pub fn start(
		&mut self,
		state_mutex: Arc<Mutex<ApplicationState>>,
		downloads_path: PathBuf,
		num_threads: usize,
	) {
		println!("Starting download: {} -> {}", self.url, self.file_name);

		// Create a new event handler
		let event_handler = Arc::new(Mutex::new(DownloaderEventHandler::new()));

		let file_path = downloads_path.join(&self.file_name);

		// Create a new downloader
		let downloader = Downloader::new(
			self.url.clone(),
			file_path.into_os_string().into_string().unwrap(),
			event_handler.clone(),
			self.md5.clone(),
			num_threads,
		);

		// Clear the error field
		self.error = None;

		// Start the download
		let download_handle = tokio::spawn(async move {
			downloader.start().await.unwrap();
		});

		// Handle events
		// let event_handler_clone = event_handler.clone();

		// let download_url = self.clone().url;
		let file_name = self.clone().file_name;
		let listen_handler = tokio::spawn(async move {
			let receiver = event_handler.lock().await.listen().await;
			let mut receiver_guard = receiver.lock().await;
			while let Some(event) = receiver_guard.recv().await {
				let mut state = state_mutex.lock().await;

				// We need to find the download in the downloads list
				let download = state
					.selected_instance_or_fail()
					.downloads
					.iter_mut()
					.find(|d| d.file_name == file_name)
					.unwrap();
				download.handle_event(event);
				download.pending_update = true;

				// let download_clone = download.clone();
				// match state.download_event_trigger {
				// 	Some(ref trigger) => {
				// 		let _ = trigger.on_download_update(download_clone);
				// 	}
				// 	None => {}
				// }
			}
		});

		// return event_handler;

		// return Ok(());
	}

	pub fn handle_event(&mut self, event: DownloaderEvent) {
		match event {
			DownloaderEvent::Progress { downloaded, total } => {
				self.status = DownloadStatus::Downloading;
				self.size_downloaded = format!("{}", downloaded);
				self.size_total = format!("{}", total);
			}
			DownloaderEvent::Merging => {
				self.status = DownloadStatus::Merging;
			}
			DownloaderEvent::Paused => {
				println!("Download paused");
			}
			DownloaderEvent::Resumed => {
				self.status = DownloadStatus::Downloading;
				println!("Download resumed");
			}
			DownloaderEvent::Failed { error } => {
				self.status = DownloadStatus::Failed;
				self.error = Some(error);
			}
			DownloaderEvent::Verifying => {
				self.status = DownloadStatus::Verifying;
			}
			DownloaderEvent::Complete => {
				println!("Download complete!");
				self.status = DownloadStatus::Downloaded;
				self.completed_at = Some(default_date());
			}
		}
	}

	pub fn delete_file(&self, downloads_path: PathBuf) -> Result<(), io::Error> {
		return file_controller::delete_file_if_exists(downloads_path.join(&self.file_name));
	}
}

#[derive(Debug)]
pub struct DownloaderSize {
	pub total: u64,
	pub downloaded: u64,
}

#[derive(Clone, Debug)]
pub struct Downloader {
	client: Client,                                    // HTTP client for making requests
	url: String,                                       // URL of the file to download
	file_path: String,                                 // Path to save the downloaded file
	md5: Option<String>,                               // MD5 hash of the file
	event_handler: Arc<Mutex<DownloaderEventHandler>>, // Event handler for sending events
	// paused: Arc<AtomicBool>, // Atomic boolean for pause/resume functionality
	num_threads: usize, // Number of threads to use for downloading
	size: Arc<Mutex<DownloaderSize>>,
}

impl Downloader {
	// Create a new Downloader instance
	pub fn new(
		url: String,
		file_path: String,
		event_handler: Arc<Mutex<DownloaderEventHandler>>,
		md5: Option<String>,
		num_threads: usize,
	) -> Self {
		Downloader {
			client: Client::new(), // Initialize the HTTP client
			url,
			file_path,
			event_handler,
			md5,
			// paused: Arc::new(AtomicBool::new(false)), // Initialize paused state
			num_threads,
			size: Arc::new(Mutex::new(DownloaderSize {
				total: 0,
				downloaded: 0,
			})),
		}
	}

	// Pause the download
	// pub fn pause(&self) {
	// 	self.paused.store(true, Ordering::SeqCst);
	// }

	// Resume the download
	// pub fn resume(&self) {
	// 	self.paused.store(false, Ordering::SeqCst);
	// }

	// Start the download process
	pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		println!("Starting download: {} -> {}", self.url, self.file_path);

		// Send initial progress event
		self.event_handler
			.lock()
			.await
			.send_event(DownloaderEvent::Progress {
				downloaded: 0,
				total: 0,
			})
			.await?;

		// Create folders if they don't exist
		let parent_dir = Path::new(&self.file_path).parent().unwrap();
		tokio::fs::create_dir_all(parent_dir).await?;

		let head_request = self.client.head(&self.url).send().await?;

		// Get the total size of the file
		let content_length_header = match head_request.headers().get("content-length") {
			Some(size) => size,
			None => return Err("Content-Length header is missing".into()),
		};

		// Print all headers
		// for (name, value) in head_request.headers().iter() {
		// 	println!("{}: {:?}", name, value);
		// }

		// Cast the content length to u64
		let total_size = content_length_header.to_str()?.parse::<u64>()?;

		// Check if server supports byte ranges
		let mut supports_range = true;
		if self.num_threads > 1
			&& head_request.headers().get("accept-ranges")
				!= Some(&HeaderValue::from_static("bytes"))
		{
			supports_range = false;
			println!("Server does not support byte ranges. Downloading with a single thread.");
		}

		// TODO: If the content length is missing, we can try to download the file in a single thread

		// Check the size is greater than 0
		if total_size <= 0 {
			return Err("File size is invalid (0 bytes)".into());
		}

		// Update the total size
		self.size.lock().await.total = total_size;

		let final_threads = if supports_range { self.num_threads } else { 1 };

		// let final_threads = 1;

		// Calculate the size of each chunk
		let chunk_size = total_size / final_threads as u64;

		// Create a vector to hold the download tasks
		let mut tasks = Vec::new();

		// Spawn a download task for each chunk
		for i in 0..final_threads {
			// Calculate the start byte of the chunk
			let start = i as u64 * chunk_size;
			// Calculate the end byte of the chunk
			let end = if i == final_threads - 1 {
				total_size
			} else {
				(i as u64 + 1) * chunk_size - 1
			};

			// Clone the URL for the task
			let url = self.url.clone();
			// Create a unique file path for the chunk
			let file_path = format!("{}.part{}", self.file_path, i);
			// Clone the HTTP client for the task
			let client = self.client.clone();
			// Clone the paused state for the task
			// let paused = self.paused.clone();
			// Clone the event handler for the task
			// let event_handler = self.event_handler.clone();

			// Spawn the download task
			let event_handler_clone = self.event_handler.clone();
			let size_clone = self.size.clone();

			let chunk_event_handler = Arc::new(Mutex::new(DownloaderChunkEventHandler::new()));
			let chunk_event_handler_clone = chunk_event_handler.clone();

			// Spawn a task to listen for chunk events
			tokio::spawn(async move {
				let reciever = chunk_event_handler.lock().await.listen().await;
				let mut reciever_guard = reciever.lock().await;

				while let Some(event) = reciever_guard.recv().await {
					match event {
						DownloaderChunkEvent::Progress {
							downloaded,
							downloaded_diff,
							total,
						} => {
							// Update the total downloaded size
							let mut size = size_clone.lock().await;
							size.downloaded += downloaded_diff;
							// size.total = total_size;

							event_handler_clone
								.lock()
								.await
								.send_event(DownloaderEvent::Progress {
									downloaded: size.downloaded,
									total: size.total,
								})
								.await;
						}
						DownloaderChunkEvent::Complete => {
							println!("Chunk download complete!");
						}
						DownloaderChunkEvent::Failed { error } => {
							event_handler_clone
								.lock()
								.await
								.send_event(DownloaderEvent::Failed { error })
								.await;
						}
					}
				}
			});

			// Spawn and add the task to the tasks vector
			tasks.push(tokio::spawn(async move {
				// Download the chunk
				println!("Downloading chunk {} -> {} - {}", i, start, end);
				let _ = download_chunk(
					url,
					file_path,
					client,
					start,
					end,
					chunk_event_handler_clone.clone(),
					supports_range,
				)
				.await;
				println!("Downloaded chunk {} -> {} - {}", i, start, end);
			}));
		}

		// Wait for all download tasks to complete
		futures::future::try_join_all(tasks).await?;

		self.event_handler
			.lock()
			.await
			.send_event(DownloaderEvent::Merging)
			.await?;

		// Merge the downloaded chunks into a single file
		self.merge_files(final_threads).await?;

		// Verify the downloaded file
		self.verify_file().await?;

		// Send the completion event
		self.event_handler
			.lock()
			.await
			.send_event(DownloaderEvent::Complete)
			.await?;
		Ok(())
	}

	// Merge the downloaded chunks into a single file
	async fn merge_files(
		&self,
		total_threads: usize,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		// Create the output file
		let mut output_file = File::create(&self.file_path).await?;

		// If there is only one chunk, rename it to the output file
		if total_threads <= 1 {
			// If there is only one chunk, rename it to the output file
			tokio::fs::rename(format!("{}.part0", self.file_path), &self.file_path).await?;
			return Ok(());
		}

		// If we have multiple chunks, merge them into the output file
		for i in 0..total_threads {
			// Get the path of the chunk
			let part_path = format!("{}.part{}", self.file_path, i);
			// Open the chunk file
			let mut part_file = File::open(&part_path).await?;
			// Create a buffer to hold the file contents
			let mut buffer = Vec::new();
			// Read the chunk file into the buffer
			part_file.read_to_end(&mut buffer).await?;
			// Write the buffer to the output file
			output_file.write_all(&buffer).await?;
			// Remove the chunk file
			tokio::fs::remove_file(part_path).await?;
		}

		// Flush the output file to ensure all data is written
		output_file.flush().await?;

		Ok(())
	}

	async fn verify_file(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let file_hash = hash_file(self.file_path.clone());

		if let Some(md5) = &self.md5 {
			if file_hash.md5_hash != *md5 {
				return Err("File hash does not match the expected MD5 hash".into());
			}
		}

		Ok(())
	}
}

// Download a chunk of the file
async fn download_chunk(
	url: String,
	file_path: String,
	client: Client,
	start: u64,
	end: u64,
	// paused: Arc<AtomicBool>,
	event_handler: Arc<Mutex<DownloaderChunkEventHandler>>,
	supports_range: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	// Create a new header map
	let mut headers = HeaderMap::new();

	// Add the range header to the request
	if supports_range {
		headers.insert(RANGE, format!("bytes={}-{}", start, end).parse()?);
	}

	// Check if the part file exists and determine the already downloaded size
	let mut downloaded = 0u64;

	// Check if the part file exists
	if Path::new(&file_path).exists() {
		// Check if the server supports byte ranges
		if supports_range {
			// If it does, open the part file and get the size
			let part_file = File::open(&file_path).await?;
			downloaded = part_file.metadata().await?.len();
			if downloaded > 0 {
				// Add the range header to resume the download
				headers.insert(
					RANGE,
					format!("bytes={}-{}", start + downloaded, end).parse()?,
				);
			}
		} else {
			// If it doesn't, delete the old part file
			tokio::fs::remove_file(file_path.clone()).await?;
		}
	}

	// Send the HTTP request
	let response = client.get(&url).headers(headers).send().await?;
	// Calculate the total size of the chunk
	let total_size = end - start + 1;

	// Get the response body as a stream
	let mut stream = response.bytes_stream();
	let mut file = if downloaded > 0 {
		// Open the part file for appending if it exists
		let std_file = OpenOptions::new().append(true).open(&file_path)?;
		// Convert std::fs::File to tokio::fs::File
		File::from_std(std_file)
	} else {
		// Create a new part file if it doesn't exist
		File::create(&file_path).await?
	};

	// Read the response stream in chunks
	while let Some(chunk) = stream.next().await {
		// Check if the download is paused
		// if paused.load(Ordering::SeqCst) {
		// 	// Send a paused event
		// 	event_handler.lock().await.send_event(DownloaderEvent::Paused).await?;
		// 	// Wait while the download is paused
		// 	while paused.load(Ordering::SeqCst) {
		// 		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		// 	}
		// 	// Send a resumed event
		// 	event_handler.lock().await.send_event(DownloaderEvent::Resumed).await?;
		// }

		// Get the chunk
		let chunk = chunk?;
		// Write the chunk to the file
		file.write_all(&chunk).await?;
		// Update the downloaded size
		downloaded += chunk.len() as u64;

		// Send a progress event
		event_handler
			.lock()
			.await
			.send_event(DownloaderChunkEvent::Progress {
				downloaded,
				downloaded_diff: chunk.len() as u64,
				total: total_size,
			})
			.await?;
	}

	// Flush the file to ensure all data is written
	file.flush().await?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use rand::Rng;
	// use warp::Filter;
	use reqwest::header::{HeaderValue, ACCEPT_RANGES, CONTENT_LENGTH};
	use std::os::unix::net::SocketAddr;
	use std::sync::Arc;
	use std::time::Duration;
	use tempfile::tempdir;
	use tokio::io::AsyncReadExt;
	use tokio::sync::Mutex;
	use tokio::time::sleep;
	use tokio::time::timeout;
	use warp::hyper::Body;
	use warp::{http::Response, Filter};

	#[tokio::test]
	async fn test_downloader() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		// Set up a temporary directory for the download
		let temp_dir = tempdir()?;
		let file_path = temp_dir.path().join("downloaded_file.txt");
		let file_path_str = file_path.to_str().unwrap().to_string();

		// Generate a large random file for testing
		let file_size_mb = 10; // Size of the file in megabytes
		let mut rng = rand::thread_rng();
		let file_content: Vec<u8> = (0..file_size_mb * 1024 * 1024)
			.map(|_| rng.gen::<u8>())
			.collect();

		let file_content_clone = file_content.clone();
		let file_content_clone2 = file_content.clone();

		let get_route = warp::get().and(warp::path("largefile")).map(move || {
			let file_content = file_content_clone.clone();
			// let content_type = content_type.clone();
			warp::http::Response::builder()
				.header("Content-Type", "text/plain")
				.header("Content-Length", file_content_clone.len())
				.body(file_content.to_vec())
		});

		// Route to handle HEAD requests
		let head_route = warp::path("largefile").and(warp::head()).map(move || {
			let file_content = file_content_clone2.clone();
			// let content_type = content_type.clone();

			warp::http::Response::builder()
				.header("Content-Type", "text/plain")
				.header("Content-Length", file_content.len())
				.body(Vec::new())
		});

		let (addr, server) =
			warp::serve(get_route.or(head_route)).bind_ephemeral(([127, 0, 0, 1], 0));
		tokio::spawn(server);

		let url = format!("http://{}/largefile", addr);
		let num_threads = 4; // Number of threads to use for downloading

		let event_handler = Arc::new(Mutex::new(DownloaderEventHandler::new())); // Create a new event handler
		let downloader = Downloader::new(
			url.clone(),
			file_path_str.clone(),
			event_handler.clone(),
			None,
			num_threads,
		); // Create a new downloader

		// Spawn the download task
		let download_handle = tokio::spawn(async move {
			downloader.start().await.unwrap();
		});

		// Listen for events with a timeout
		let event_handler_clone = event_handler.clone();
		let listen_handle = tokio::spawn(async move {
			let receiver = event_handler_clone.lock().await.listen().await;
			let mut receiver_guard = receiver.lock().await;
			while let Some(event) = receiver_guard.recv().await {
				match event {
					DownloaderEvent::Verifying {} => {
						// println!("Downloaded: {} / {}", downloaded, total);
					}
					DownloaderEvent::Progress { downloaded, total } => {
						println!("Downloaded: {} / {}", downloaded, total);
					}
					DownloaderEvent::Merging => {
						println!("Download merged successfully!");
					}
					DownloaderEvent::Paused => {
						println!("Download paused");
					}
					DownloaderEvent::Resumed => {
						println!("Download resumed");
					}
					DownloaderEvent::Failed { error } => {
						println!("Download failed: {}", error);
					}
					DownloaderEvent::Complete => {
						println!("Download complete!");
						break;
					}
				}
			}
		});

		// Wait for the download to complete
		timeout(Duration::from_secs(30), download_handle).await??;

		// Wait for the event listener to finish
		timeout(Duration::from_secs(5), listen_handle).await??;

		// Verify the downloaded file
		let mut downloaded_file = File::open(file_path).await?;
		let mut downloaded_content = Vec::new();
		downloaded_file.read_to_end(&mut downloaded_content).await?;
		assert_eq!(downloaded_content, file_content);

		Ok(())
	}

	async fn test_multi_threaded_download(
		num_threads: usize,
		addr: String,
		file_data: Vec<u8>,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		// Create a temporary directory
		let temp_dir = tempdir()?;
		let file_path = temp_dir
			.path()
			.join(format!("downloaded_file_mt_{}", num_threads));
		let file_path_str = file_path.to_str().unwrap().to_string();

		let event_handler = Arc::new(Mutex::new(DownloaderEventHandler::new())); // Create a new event handler

		// Create a Downloader instance
		let downloader = Downloader::new(
			format!("http://{}/multi-threaded-file", addr),
			file_path_str.clone(),
			event_handler,
			None,
			num_threads,
		);

		// Start the download
		downloader.start().await?;

		// Verify the file content
		let mut downloaded_file = File::open(file_path).await?;
		let mut downloaded_data = Vec::new();
		downloaded_file.read_to_end(&mut downloaded_data).await?;
		assert_eq!(downloaded_data, file_data);

		Ok(())
	}

	#[tokio::test]
	async fn test_multi_threaded() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		// Generate a large random file for testing
		let file_size_mb = 10; // Size of the file in megabytes
		let mut rng = rand::thread_rng();
		let file_data: Vec<u8> = (0..file_size_mb * 1024 * 1024)
			.map(|_| rng.gen::<u8>())
			.collect();

		// Create mock data for the file
		// let file_data = b"Hello, world! This is a test file for multi-threaded download.".to_vec();

		// Set up a mock server
		let file_data_clone = file_data.clone();
		let routes = warp::path!("multi-threaded-file")
			.and(warp::method())
			.and(warp::header::optional::<String>("range"))
			.map(move |method: warp::http::Method, range: Option<String>| {
				let file_data = file_data_clone.clone();
				match method {
					warp::http::Method::GET => {
						let mut response = Response::new(Body::from(file_data.clone()));
						let mut content_length = file_data.len().to_string();

						if let Some(range_header) = range {
							let parts: Vec<&str> = range_header.split('=').collect();
							if parts.len() == 2 {
								let ranges: Vec<&str> = parts[1].split('-').collect();
								if ranges.len() == 2 {
									let start: usize = ranges[0].parse().unwrap_or(0);
									let end: usize =
										ranges[1].parse().unwrap_or(file_data.len() - 1);
									let sliced_data = file_data
										[start..=std::cmp::min(end, file_data.len() - 1)]
										.to_vec();
									*response.body_mut() = Body::from(sliced_data.clone());
									content_length = sliced_data.len().to_string();
								}
							}
							response
								.headers_mut()
								.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
							response.headers_mut().insert(
								CONTENT_LENGTH,
								HeaderValue::from_str(content_length.as_str()).unwrap(),
							);
						}
						response
					}
					warp::http::Method::HEAD => {
						let mut response = Response::new(Body::empty());
						response
							.headers_mut()
							.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
						response.headers_mut().insert(
							CONTENT_LENGTH,
							HeaderValue::from_str(&file_data.len().to_string()).unwrap(),
						);
						response
					}
					_ => Response::builder().status(405).body(Body::empty()).unwrap(),
				}
			});

		// Start the mock server
		let (addr, server) =
			warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async {
				tokio::signal::ctrl_c().await.unwrap();
			});

		let server_handle = tokio::spawn(server);

		// Give the server some time to start
		sleep(Duration::from_millis(100)).await;

		// Run the test in multiple thread counts
		test_multi_threaded_download(1, addr.clone().to_string(), file_data.clone()).await?;
		test_multi_threaded_download(2, addr.clone().to_string(), file_data.clone()).await?;
		test_multi_threaded_download(3, addr.clone().to_string(), file_data.clone()).await?;
		test_multi_threaded_download(4, addr.clone().to_string(), file_data.clone()).await?;
		test_multi_threaded_download(8, addr.clone().to_string(), file_data.clone()).await?;

		// Shutdown the server
		server_handle.abort();

		Ok(())
	}
}
