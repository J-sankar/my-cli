use clap::Parser;

mod commands;
use crate::commands::{Commands, handle_create};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Create(create_args) => {
            if let Err(error) = handle_create(create_args) {
                eprintln!("error:{error}");
                std::process::exit(1);
            }
        }
    }
}
