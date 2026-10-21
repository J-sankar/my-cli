use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use rand::RngCore;
use sha2::{Digest, Sha256};
use url::Url;
pub struct AuthURLSchema {
    auth_uri: String,
    scope: Vec<String>,
    response_type: String,
    pub state: String,
    redirect_url: String,
    client_id: String,
    code_challenge: String,
    code_challenge_method: String,
}
impl AuthURLSchema {
    pub fn new(client_id: &String, redirect_url: &String, auth_uri: &String) -> Self {
        Self {
            auth_uri: auth_uri.to_string(),
            scope: Vec::new(),
            response_type: String::from("code"),
            state: String::new(),
            redirect_url: redirect_url.to_string(),
            client_id: client_id.to_string(),
            code_challenge: String::new(),
            code_challenge_method: String::new(),
        }
    }

    pub fn add_scope(&mut self, scope: &str) {
        self.scope.push(scope.to_string())
    }

    pub fn add_pkce_config(&mut self, code_challenge: String, method: &str) {
        self.code_challenge = code_challenge;
        self.code_challenge_method = method.to_string()
    }

    pub fn add_state(&mut self) {
        let state = generate_state();
        self.state = state;
    }

    pub fn build_auth_url(&self) -> Result<Url, url::ParseError> {
        let scope = self.scope.join(" ");
        Url::parse_with_params(
            &self.auth_uri, // &str or String
            &[
                ("client_id", self.client_id.as_str()),
                ("redirect_uri", self.redirect_url.as_str()),
                ("response_type", self.response_type.as_str()),
                ("scope", scope.as_str()),
                ("code_challenge", self.code_challenge.as_str()),
                ("code_challenge_method", self.code_challenge_method.as_str()),
                ("state", self.state.as_str()),
                ("access_type", "offline"),
            ],
        )
    }
}

fn generate_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);

    URL_SAFE_NO_PAD.encode(bytes)
}

fn code_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

pub fn get_code_challenge_and_verifier() -> (String, String) {
    let verifier = generate_verifier();
    let challenge = code_challenge(&verifier);
    (challenge, verifier)
}
fn generate_state() -> String {
    let mut bytes = [0u8; 16]; // 128 bits is plenty for state
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
