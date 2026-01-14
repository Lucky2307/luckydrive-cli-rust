use std::io::{Error, ErrorKind};

use crate::config::{self, API_ENDPOINT, SERVICE_NAME};
use crate::token::get_token;

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
pub fn login(code: &str) -> Result<String, Error> {
    let client = Client::new();
    // let username = config::load_username().unwrap();
    // let entry = Entry::new(&service_name, &username).unwrap();
    // if let Ok(_) = entry.get_password() {
    //   return Err("Already logged in".to_string())
    // }
    match get_token() {
        Ok(_) => return Err(Error::new(ErrorKind::AlreadyExists, "Already logged in")),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {}
            other_error => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Error getting current user: {}", other_error),
                ));
            }
        },
    }

    let response = client
        .post(format!("{}/api/login", *API_ENDPOINT))
        .json(&serde_json::json!({ "code": code }))
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
            let entry = Entry::new(&*SERVICE_NAME, &json_body.user_name).unwrap();
            entry.set_password(&json_body.user_token).unwrap();
            config::save_username(&json_body.user_name).unwrap();
            Ok(format!("Logged in as {}", &json_body.user_name))
        }
        StatusCode::NOT_FOUND => {
            return Err(Error::new(ErrorKind::NotFound, "Invalid credentials"));
        }
        e => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}
