use std::env::var;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: String,
}

pub fn config_path() -> PathBuf {
    let path = Path::new("./.config").to_path_buf();
    if !path.exists() {
        let default_config = Config {
            username: "".to_string(),
        };
        fs::write(&path, serde_json::to_string_pretty(&default_config).unwrap()).unwrap();
    }; 
    path
}

pub fn save_username(username: &str) -> Result<(), String> {
    let config = Config {
        username: username.to_string(),
    };

    let path = config_path();
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_username() -> Result<String, String> {
    let path = config_path();
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: Config = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(config.username)
}

pub static API_ENDPOINT: LazyLock<String> = LazyLock::new(|| {var("API_ENDPOINT").expect("API_ENDPOINT not set")});
pub static SERVICE_NAME: LazyLock<String> = LazyLock::new(|| {var("SERVICE_NAME").expect("SERVICE_NAME not set")});