use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UploadUrl {
    video_id: String,
    video_upload_url: String,
    thumbnail_upload_url: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UploadRequestPayload {
    video_size: u32,
    thumbnail_size: u32,
}

fn get_upload_url(token: &str, video_size: u32, thumbnail_size: u32) -> Result<UploadUrl, String> {
    let client = Client::new();
    let payload = UploadRequestPayload {
        video_size: video_size,
        thumbnail_size: thumbnail_size,
    };

    let response = client
        .post(format!("{}/api/upload", *config::API_ENDPOINT))
        .header("token", token)
        .json(&serde_json::json!(payload))
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                "Request timed out".to_string()
            } else if e.is_connect() {
                "Cannot connect to server".to_string()
            } else {
                format!("Error: {}", e)
            }
        })?;

    match response.status() {
      StatusCode::OK => {
          let raw_body = response.text().unwrap_or_default();
          serde_json::from_str::<UploadUrl>(&raw_body).map_err(|e| format!("Failed to unwrap JSON: {}", e))
      }
      e => return Err(format!("Failed requesting upload url: {}", e))
    }
}

// pub fn upload(file_path: &str) -> Result<String, String> {
  
//   let upload_url = get_upload_url(token, video_size, thumbnail_size);
// }
