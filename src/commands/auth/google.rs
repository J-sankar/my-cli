use serde::Deserialize;
use std::path::Path;

use super::AuthAction;

#[derive(Debug, Deserialize)]
struct OAuthClientFile {
    installed: GoogleClientConfig,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
pub struct GoogleClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_uri: String,
    pub token_uri: String,
}

pub fn load_client_config() -> Result<GoogleClientConfig, String> {
    let path = Path::new("credentials/client_secret.json");
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let credentials: OAuthClientFile =
        serde_json::from_reader(&file).map_err(|error| error.to_string())?;
    Ok(credentials.installed)
}

pub fn validate_client_config(client_config: &GoogleClientConfig) -> Result<(), String> {
    if client_config.auth_uri.trim().is_empty()
        || client_config.client_id.trim().is_empty()
        || client_config.client_secret.trim().is_empty()
        || client_config.token_uri.trim().is_empty()
    {
        return Err(
            "google client credentials not valid. Did you overwrite any files?".to_string(),
        );
    }

    Ok(())
}

pub fn handle_google(action: &AuthAction) -> Result<(), String> {
    match &action {
        AuthAction::Login => {
            let client_config = load_client_config()?;
            validate_client_config(&client_config)?;
            println!("Google OAuth client configuration loaded.");

        }
        AuthAction::Status => {
            println!("Google authentication status is not implemented yet.");
        }

        AuthAction::Logout => {
            println!("Google logout is not implemented yet.");
        }
    }

    Ok(())
}
