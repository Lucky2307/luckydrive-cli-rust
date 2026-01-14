use std::env::var;

use crate::config;

use keyring::Entry;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    user_name: String,
    user_token: String,
}

// TODO: Handle error properly
pub fn login(code: &str) -> Result<String, String> {
    let client = Client::new();
    let api = var("API_ENDPOINT").expect("API_ENDPOINT not set");
    let service_name = var("SERVICE_NAME").expect("SERVICE_NAME not set");
    let username = config::load_username().unwrap();
    let entry = Entry::new(&service_name, &username).unwrap();
    if let Ok(_) = entry.get_password() {
      return Err("Already logged in".to_string())
    }

    let response = client
        .post(format!("{}/api/login", &api))
        .json(&serde_json::json!({ "code": code }))
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
            let json_body = serde_json::from_str::<LoginResponse>(&raw_body).unwrap();
            let entry = Entry::new(&service_name, &json_body.user_name).unwrap();
            entry.set_password(&json_body.user_token).unwrap();
            config::save_username(&json_body.user_name).unwrap();
            Ok(format!("Logged in as {}", &json_body.user_name))
        }
        StatusCode::NOT_FOUND => {
            Err("User not found".to_string())
        }
        e => Err(e.to_string()),
    }
}
