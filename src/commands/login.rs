use std::env::var;

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
pub fn login(code: &str) {
    let client = Client::new();
    let api = var("API_ENDPOINT").expect("API_ENDPOINT not set");

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
        })
        .unwrap();

    match response.status() {
        StatusCode::OK => {
            let raw_body = response.text().unwrap_or_default();
            let json_body = serde_json::from_str::<LoginResponse>(&raw_body).unwrap();
            let service_name = var("SERVICE_NAME").expect("SERVICE_NAME not set");
            let entry = Entry::new(&service_name, &json_body.user_name).unwrap();
            entry.set_password(&json_body.user_token).unwrap();
        }
        StatusCode::NOT_FOUND => {
            println!("User not found")
        }
        e => println!("Error: {}", e),
    }
}
