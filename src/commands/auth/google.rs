use core::fmt;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;
use std::time::Duration;

use reqwest::{self};

use super::AuthAction;
use crate::commands::auth::callback;
use crate::commands::auth::utils::{AuthURLSchema, get_code_challenge_and_verifier};
use crate::utils::store_token;
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

#[derive(Deserialize)]
pub struct GoogleOAuthTokenResponse {
    expires_in: u64,
    access_token: String,
    refresh_token: String,
    scope: String,
    token_type: String,
}
#[derive(Deserialize)]
pub struct GoogleRefreshResponse {
    expires_in: u64,
    access_token: String,
    scope: String,
    token_type: String,
}

impl fmt::Display for GoogleRefreshResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "access_token: redacted, expires in: {} , token_type: {}",
            self.expires_in, self.token_type
        )
    }
}

impl GoogleRefreshResponse {
    pub fn get_access_token(&self) -> String {
        self.access_token.to_string()
    }
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

pub fn handle_oauth_flow(client_config: &GoogleClientConfig) -> Result<(), String> {
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

    if !status.is_success() {
        let error_body = res.text().unwrap_or_default();

        return Err(format!(
            "Google token exchange failed ({status}): {error_body}"
        ));
    }

    let body: GoogleOAuthTokenResponse = res
        .json()
        .map_err(|error| format!("Could not read Google token response: {error}"))?;

    println!("Login successful.");
    if body.access_token.is_empty() || body.refresh_token.is_empty() {
        return Err("no tokens found".to_string());
    }

    store_token(&body.refresh_token)?;
    Ok(())
}

#[derive(Debug)]
pub struct Unauthorized;

impl fmt::Display for Unauthorized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unauthorized: token missing, inalid or expired")
    }
}

impl Error for Unauthorized {}

pub fn get_new_token(refresh_token: &str) -> Result<GoogleRefreshResponse, Box<dyn Error>> {
    let http_client = reqwest::blocking::Client::new();
    let client_config = load_client_config()?;
    let token_uri = &client_config.token_uri;
    let req_body = [
        ("client_id", &client_config.client_id),
        ("client_secret", &client_config.client_secret),
        ("grant_type", &"refresh_token".to_string()),
        ("refresh_token", &refresh_token.to_string()),
    ];
    let res = http_client
        .post(token_uri)
        .form(&req_body)
        .send()
        .map_err(|err| err.to_string())?;
    let status = res.status();
    if !status.is_success() {
        let error_body = res.text().unwrap_or_default();
        if (status == reqwest::StatusCode::BAD_REQUEST && error_body.contains("invalid_grant"))
            || status == reqwest::StatusCode::UNAUTHORIZED
        {
            return Err(Unauthorized.into());
        }
        if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            return Err("some error occured from googles side, please try again".into());
        }
        return Err(format!("Google token refresh failed ({status}): {error_body}").into());
    }
    let body: GoogleRefreshResponse = res.json().map_err(|err| err.to_string())?;
    let google_token_response = GoogleRefreshResponse {
        access_token: body.access_token,
        expires_in: body.expires_in,
        scope: body.scope,
        token_type: body.token_type,
    };
    Ok(google_token_response)
}
