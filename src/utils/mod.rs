use std::error::Error;

use keyring::{Entry, Error::NoDefaultStore};

const SERVICE: &str = "mc";
const ACCOUNT: &str = "google-refresh-token";

fn get_entry() -> Result<Entry, String> {
    Entry::new(SERVICE, ACCOUNT).map_err(|err| match err {
        NoDefaultStore => {
            "No secure credential store is available. Start or unlock your system keyring."
                .to_string()
        }
        other => format!("Could not access secure credential storage: {other}"),
    })
}

pub fn store_token(password: &str) -> Result<(), String> {
    let entry = get_entry()?;
    match entry.set_password(password) {
        Ok(()) => Ok(()),
        Err(_) => Err("could not store tokens".to_string()),
    }
}

pub fn load_token() -> Result<String, Box<dyn Error>> {
    let entry = get_entry()?;
    match entry.get_password() {
        Ok(token) => Ok(token),
        Err(err) => Err(err.into()),
    }
}
