use std::io::{Error, ErrorKind};

use crate::config::{self, API_ENDPOINT, SERVICE_NAME};

use keyring::Entry;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    username: String,
    access_token: String,
}

// TODO: Handle error properly
pub fn login(username: &str, password: &str) -> Result<String, Error> {
    let client = Client::new();

    let response = client
        .post(format!("{}/api/login", *API_ENDPOINT))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                Error::new(ErrorKind::TimedOut, "Request timed out")
            } else if e.is_connect() {
                Error::new(ErrorKind::NotConnected, "Cannot connect to server")
            } else {
                Error::new(ErrorKind::Other, format!("Error: {}", e))
            }
        })?;

    match response.status() {
        StatusCode::OK => {
            let raw_body = response.text().unwrap_or_default();
            let json_body = serde_json::from_str::<LoginResponse>(&raw_body).unwrap();
            let entry = Entry::new(&*SERVICE_NAME, &json_body.username).unwrap();
            entry.set_password(&json_body.access_token).unwrap();
            config::save_username(&json_body.username).unwrap();
            Ok(format!("Logged in as {}", &json_body.username))
        }
        StatusCode::NOT_FOUND => {
            return Err(Error::new(ErrorKind::NotFound, "Invalid credentials"));
        }
        e => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}
