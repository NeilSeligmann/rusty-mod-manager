use std::sync::Arc;

// src/events.rs
use tokio::sync::mpsc::{self, Sender, Receiver};
use serde::Serialize;
use tokio::sync::{oneshot, Mutex};

#[derive(Debug, Serialize)]
pub enum DownloaderEvent {
	Progress { downloaded: u64, total: u64 },
	Paused,
	Resumed,
	Complete,
	Merging,
	Verifying,
	Failed { error: String },
}

#[derive(Debug)]
pub struct DownloaderEventHandler {
	sender: Sender<DownloaderEvent>,
	receiver: Arc<Mutex<Receiver<DownloaderEvent>>>,
}

impl DownloaderEventHandler {
	pub fn new() -> Self {
		let (sender, receiver) = mpsc::channel(100);
		DownloaderEventHandler {
			sender,
			receiver: Arc::new(Mutex::new(receiver)),
		}
	}

	pub async fn send_event(&self, event: DownloaderEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		self.sender.send(event).await?;
		Ok(())
	}

	pub async fn listen(&self) -> Arc<Mutex<Receiver<DownloaderEvent>>> {
		self.receiver.clone()
	}
}


#[derive(Debug)]
pub enum DownloaderChunkEvent {
	Progress { downloaded: u64, downloaded_diff: u64, total: u64 },
	Complete,
	Failed { error: String },
}

#[derive(Debug)]
pub struct DownloaderChunkEventHandler {
	sender: Sender<DownloaderChunkEvent>,
	receiver: Arc<Mutex<Receiver<DownloaderChunkEvent>>>,
}

impl DownloaderChunkEventHandler {
	pub fn new() -> Self {
		let (sender, receiver) = mpsc::channel(100);
		DownloaderChunkEventHandler {
			sender,
			receiver: Arc::new(Mutex::new(receiver)),
		}
	}

	pub async fn send_event(&self, event: DownloaderChunkEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		self.sender.send(event).await?;
		Ok(())
	}

	pub async fn listen(&self) -> Arc<Mutex<Receiver<DownloaderChunkEvent>>> {
		self.receiver.clone()
	}
}
