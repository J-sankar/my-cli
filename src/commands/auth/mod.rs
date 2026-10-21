use clap::Subcommand;
use std::fmt;

mod callback;
mod google;
pub mod utils;

#[derive(Debug, Subcommand)]
pub enum AuthProvider {
    ///authenticate with google
    #[command(subcommand)]
    Google(AuthAction),

    ///authenticate with github
    #[command(subcommand)]
    Github(AuthAction),
}

#[derive(Debug, Subcommand)]
pub enum AuthAction {
    ///login with your provider
    Login,

    ///logout and remove stored credentials
    Logout,

    ///check authentication status for the selected provider
    Status,
}

impl std::fmt::Display for AuthAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            AuthAction::Login => write!(f, "login"),
            AuthAction::Logout => write!(f, "logout"),
            AuthAction::Status => write!(f, "status"),
        }
    }
}

pub fn handle_auth(auth_client: &AuthProvider) -> Result<(), String> {
    match auth_client {
        AuthProvider::Google(action) => {
            google::handle_google(action)?;
            Ok(())
        }
        AuthProvider::Github(action) => {
            println!("{} not implemented", action);
            Ok(())
        }
    }
}
