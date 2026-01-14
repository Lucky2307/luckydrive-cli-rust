use std::io::{Error, ErrorKind};

use keyring::Entry;

use crate::config::{self, SERVICE_NAME, save_username};

fn get_entry() -> Result<Entry, Error> {
    let username = config::load_username().map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(Entry::new(&SERVICE_NAME, &username).map_err(|e| Error::new(ErrorKind::Other, e))?)
}

pub fn get_token() -> Result<String, Error> {
    let entry = get_entry()?;
    entry
        .get_password()
        .map_err(|e| Error::new(ErrorKind::NotFound, e))
}

pub fn delete_token() -> Result<(), Error> {
    let entry = get_entry()?;
    entry
        .delete_credential()
        .map_err(|e| Error::new(ErrorKind::NotFound, e))?;
    save_username("").map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(())
}
