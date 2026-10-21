use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

use reqwest::{self};

use crate::commands::auth::callback;
use crate::commands::auth::utils::{AuthURLSchema, get_code_challenge_and_verifier};

use super::AuthAction;
const GOOGLE_CLIENT_CONFIG_PATH: &str = "credentials/client_secret.json";
#[derive(Deserialize)]
struct OAuthClientFile {
    installed: GoogleClientConfig,
}

#[derive(Deserialize)]
pub struct GoogleClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub redirect_uris: Vec<String>,
}

pub fn load_client_config() -> Result<GoogleClientConfig, String> {
    let path = Path::new(GOOGLE_CLIENT_CONFIG_PATH);
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
        || client_config.redirect_uris.is_empty()
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
            handle_oauth_flow(&client_config)?;
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

fn handle_oauth_flow(client_config: &GoogleClientConfig) -> Result<(), String> {
    let (listener, port) = callback::bind_listener()?;
    println!("{port}");

    let redirect_url = format!("{}:{}/callback", client_config.redirect_uris[0], port);
    println!("{redirect_url}");
    let (pkce_challenge, pkce_verifier) = get_code_challenge_and_verifier();

    // Generate the full authorization URL.
    let mut auth_schema = AuthURLSchema::new(
        &client_config.client_id,
        &redirect_url,
        &client_config.auth_uri,
    );
    auth_schema.add_scope("https://www.googleapis.com/auth/forms.body");
    auth_schema.add_state();
    auth_schema.add_pkce_config(pkce_challenge, "S256");

    let auth_url = auth_schema
        .build_auth_url()
        .map_err(|error| error.to_string())?;

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browsing to: {}", auth_url);
    if webbrowser::open(auth_url.as_ref()).is_err() {
        eprintln!("failed to open default browser, click manually on the url ")
    }
    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| error.to_string())?;

    //synchronous callback server returns the code and state
    let callback_data: callback::CallbackData =
        callback::wait_for_code(&listener, &auth_schema.state, Duration::from_secs(120))?;

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", &client_config.client_id),
        ("client_secret", &client_config.client_secret),
        ("code", &callback_data.code),
        ("code_verifier", &pkce_verifier),
        ("redirect_uri", redirect_url.as_str()),
    ];
    let token_url = &client_config.token_uri;

    //request for tokens

    let res = http_client
        .post(token_url)
        .form(&params)
        .send()
        .map_err(|error| error.to_string())?;
    let status = res.status();
    let body: serde_json::Value = res.json().map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(format!("token exchange failed ({status}): {}", body));
    } else {
        println!("login successfull");
    }

    Ok(())
}
