use std::io::{Error, ErrorKind};

use crate::token::delete_token;

pub fn logout() -> Result<String, Error> {
    delete_token().map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::new(ErrorKind::NotFound, "Account not found")
        } else {
            Error::new(ErrorKind::Other, e)
        }
    })?;
    Ok(format!("Logged out"))
}
