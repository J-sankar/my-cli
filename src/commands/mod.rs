use clap::{Subcommand};

use crate::commands::create::{CreateCommands,handle_create_form};
pub mod create;


#[derive(Subcommand)]
pub enum Commands {
    ///create something new
    #[command(subcommand)]
    Create(create::CreateCommands),
}


pub fn handle_create(create_args:&CreateCommands) -> Result<(),String> {
    match create_args {
        CreateCommands::Form(filename) => handle_create_form(filename)
    }
}