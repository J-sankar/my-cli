use clap::Subcommand;

pub mod auth;
pub mod create;

#[derive(Subcommand)]
pub enum Commands {
    ///create something new
    #[command(subcommand)]
    Create(create::CreateCommands),

    ///used to authentication with your providers
    #[command(subcommand)]
    Auth(auth::AuthProvider),
}

pub fn handle_create(create_args: &create::CreateCommands) -> Result<(), String> {
    match create_args {
        create::CreateCommands::Form(filename) => create::handle_create_form(filename),
    }
}
