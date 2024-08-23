// const fn _default_none<T>() -> Option<T> {
// 	None
// }

use std::fmt::Display;

use chrono::{DateTime, Utc};
use url::{Url, ParseError};
use reqwest::header;
use tauri::http::Uri;
use urlencoding::decode;
// use time::{format_description, Time};

#[taurpc::ipc_type]
pub struct NexusModsValidateResponse {
	user_id: u32,
	key: String,
	name: String,
	email: String,
	profile_url: String,
	is_premium: bool,
	is_supporter: bool,
}

#[derive(Default)]
#[taurpc::ipc_type]
pub struct RateLimit {
	#[serde(default = "Option::default")]
	pub hourly_limit: Option<u32>,
	#[serde(default = "Option::default")]
	pub hourly_remaining: Option<u32>,
	#[serde(default = "Option::default")]
	pub hourly_reset_timestamp: Option<String>,
	#[serde(default = "Option::default")]
	pub daily_limit: Option<u32>,
	#[serde(default = "Option::default")]
	pub daily_remaining: Option<u32>,
	#[serde(default = "Option::default")]
	pub daily_reset_timestamp: Option<String>,
}

#[derive(Default)]
#[taurpc::ipc_type]
pub struct NexusModsConfig {
	#[serde(default)]
	pub api_key: Option<String>,
	#[serde(default)]
	pub user_data: Option<NexusModsValidateResponse>,
	#[serde(default)]
	pub rate_limit: RateLimit,
}

#[taurpc::ipc_type]
pub struct NMSchemeParameters {
	pub game_domain: String,
	pub mod_id: String,
	pub file_id: String,
	pub key: Option<String>,
	pub expires: Option<String>,
}

#[taurpc::ipc_type]
pub struct NMCDNOptionsResponse {
	name: String,
	short_name: String,
	URI: String,
}

#[taurpc::ipc_type]
pub struct NMDownloadUrl {
	pub url: String,
	pub filename: String,
	pub md5: Option<String>,
	pub file_request: NMSchemeParameters,
}

fn parse_date_to_unix_timestamp(timestamp: String) -> Option<String> {
	match DateTime::parse_from_str(timestamp.as_str(), "%Y-%m-%d %H:%M:%S%.3f %z") {
		Ok(v) => Some(v.with_timezone(&Utc).timestamp().to_string()),
		Err(_) => None,
	}
}

impl NexusModsConfig {
	pub fn new() -> Self {
		return Self {
			api_key: None,
			user_data: None,
			rate_limit: Default::default()
		};
	}

	pub fn get_client(&self) -> Result<reqwest::Client, reqwest::Error>{
		let mut headers = header::HeaderMap::new();

		// Insert API Key
		match self.user_data {
			Some(ref user_data) => {
				headers.insert("apikey", reqwest::header::HeaderValue::from_str(user_data.key.as_str()).unwrap());
			},
			None => {
			}
		}

		return reqwest::Client::builder()
			.default_headers(headers)
			.build();
	}

	fn extract_api_rate_limit (&mut self, headers: &reqwest::header::HeaderMap) -> RateLimit {
		// Hourly limits
		let rate_hourly_limit = headers.get("X-RL-Hourly-Limit").map(|v| v.to_str().unwrap().parse::<u32>().unwrap());
		let rate_hourly_remaining = headers.get("X-RL-Hourly-Remaining").map(|v| v.to_str().unwrap().parse::<u32>().unwrap());
		let rate_hourly_reset_str = headers.get("X-RL-Hourly-Reset").map(|v| String::from(v.to_str().unwrap()));
		// Daily limits
		let rate_daily_limit = headers.get("X-RL-Daily-Limit").map(|v| v.to_str().unwrap().parse::<u32>().unwrap());
		let rate_daily_remaining = headers.get("X-RL-Daily-Remaining").map(|v| v.to_str().unwrap().parse::<u32>().unwrap());
		let rate_daily_reset_str = headers.get("X-RL-Daily-Reset").map(|v| String::from(v.to_str().unwrap()));

		// 2024-05-28 20:38:34 +0000
		// %Y-%m-%d %H:%M:%S%.3f %z
		let hourly_reset_timestamp = match rate_hourly_reset_str {
			Some(v) => parse_date_to_unix_timestamp(v),
			None => None,
		};
		let daily_reset_timestamp = match rate_daily_reset_str {
			Some(v) => parse_date_to_unix_timestamp(v),
			None => None,
		};

		let rate_limit = RateLimit {
			hourly_limit: rate_hourly_limit,
			hourly_remaining: rate_hourly_remaining,
			hourly_reset_timestamp,
			daily_limit: rate_daily_limit,
			daily_remaining: rate_daily_remaining,
			daily_reset_timestamp,
		};

		self.rate_limit = rate_limit.clone();

		return rate_limit;
	}

	pub async fn validate_api_key(&mut self) -> Result<NexusModsValidateResponse, String> {
		if !self.api_key.is_some() {
			self.user_data = None;
			return Err("API key is not set".to_string());
		}

		let url = format!("https://api.nexusmods.com/v1/users/validate.json");
		let client = reqwest::Client::new();
		let response = client
			.get(&url)
			.header("apikey", self.api_key.clone().unwrap())
			.send()
			.await
			.map_err(|e| format!("Failed to send request: {}", e))?;

		// Extract rate limit
		self.extract_api_rate_limit(&response.headers());

		self.user_data = None;

		if response.status().is_success() {
			let response = response.json::<NexusModsValidateResponse>().await.map_err(|e| format!("Failed to parse response: {}", e))?;
			self.user_data = Some(response.clone());

			return Ok(response);
		} else {
			return Err(format!("Failed to validate API key: {}", response.status()));
		}
	}

	pub async fn parse_nxm_uri(&self, url: String) -> Result<NMDownloadUrl, String> {
		if !self.api_key.is_some() {
			return Err("API key is not set".to_string());
		}

		// Example NXM Scheme URL
		// nxm://{game}/mods/{mod_id}/files/{file_id}?key={key}&expires={expires}&user_id={user_id}
		// nxm://skyrimspecialedition/mods/121323/files/510135?key=aaaBBB112233&expires=1686009809&user_id=1234567

		let uri = url.parse::<Uri>().expect("Failed to parse nxm URI");

		let path_segments = uri.path()
			.split("/")
			.filter(|s| !s.is_empty());

		// Extract the game domain
		let game_domain = uri.host().expect("Failed to get host from URL");

		// Extract the mod_id
		let mod_id = path_segments.clone().nth(1).ok_or("Failed to get mod_id from URL")?;

		// Extract the file_id
		let file_id = path_segments.clone().nth(3).ok_or("Failed to get file_id from URL")?;

		let mut query_expires: Option<String> = None;
		let mut query_key: Option<String> = None;

		// Extract the query parameters
		match uri.query() {
			Some(query) => {
				for query_param in query.split("&") {
					let split = query_param.split("=").collect::<Vec<&str>>();
					match split[0] {
						"expires" => {
							query_expires = Some(split[1].to_string());
						},
						"key" => {
							query_key = Some(split[1].to_string());
						},
						_ => {
						}
						
					}
				}
			},
			None => {
				println!("No query parameters");
			}
		}

		let file_request_parameters = NMSchemeParameters {
			game_domain: game_domain.to_string(),
			mod_id: mod_id.to_string(),
			file_id: file_id.to_string(),
			expires: query_expires,
			key: query_key,
		};

		return self.convert_nmm_request_to_url(file_request_parameters).await;
	}

	pub async fn convert_nmm_request_to_url(&self, file_request: NMSchemeParameters) -> Result<NMDownloadUrl, String> {
		if !self.api_key.is_some() {
			return Err("API key is not set".to_string());
		}

		// If we have the user data, check if all the required fields are set
		match self.user_data {
			Some(ref user_data) => {
				if !user_data.is_premium && (!file_request.key.is_some() || !file_request.expires.is_some()) {
					return Err("User is not premium, \"key\" and \"expires\" are required.".to_string());
				}
			},
			None => {
			}
		}

		let file_request_copy = file_request.clone();
		let url = format!(
			"https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json?key={}&expires={}",
			file_request_copy.game_domain,
			file_request_copy.mod_id,
			file_request_copy.file_id,
			file_request_copy.key.unwrap_or("".to_string()),
			file_request_copy.expires.unwrap_or("".to_string())
		);

		let client = reqwest::Client::new();
		let response = client
			.get(&url)
			.header("apikey", self.api_key.clone().unwrap())
			.send()
			.await
			.map_err(|e| format!("Failed to send request: {}", e))?;

		if response.status().is_success() {
			let body = response.text().await.expect("Failed to get response body");
			let parsed_response: Vec<NMCDNOptionsResponse> = serde_json::from_str(&body.to_string())
				.map_err(|e| format!("Failed to parse response: {}", e))?;

			if parsed_response.len() <= 0 {
				return Err("No download links found".to_string());
			}
			
			// TODO: Allow selecting CDN to download from

			let url = parsed_response[0].URI.clone();

			// Parse the url
			let parsed_url = Url::parse(&url).map_err(|e: ParseError| format!("Failed to parse URL: {}", e))?;

			// Extract the filename
			let filename = parsed_url.path_segments()
				.expect("Failed to get url path segments")
				.last()
				.expect("Failed to get last path segment");

			// !IMPORTANT: The MD5 hash provided does not match the file for some reason
			// Extract the md5 hash
			// let md5 = parsed_url.query_pairs()
			// 	.find(|(key, _)| key == "md5")
			// 	.map(|(_, value)| value.to_string());

			return Ok(NMDownloadUrl{
				url,
				filename: decode(filename).expect("Failed to decode filename").to_string(),
				md5: None,
				file_request,
			});
		}

		return Err(format!("Failed to process NMM link: {}", response.status()));
	}
}
