use std::env::var;

use keyring::Entry;

use crate::config;

pub fn logout() -> Result<String, String> {
    let service_name = var("SERVICE_NAME").expect("SERVICE_NAME not set");
    let username = config::load_username().unwrap();
    let entry = Entry::new(&service_name, &username).unwrap();
    entry.delete_credential().unwrap();
    Ok(format!("Logged out of {}", username))
}
