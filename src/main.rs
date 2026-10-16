use clap::{Args, Parser, Subcommand};



#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///create something new
    #[command(subcommand)]
    Create(CreateArgs),
}

#[derive(Subcommand)]
enum CreateArgs {
    /// Adds files to myapp
    Form(Filename),
}
#[derive(Args)]
struct Filename {
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Create(create_args) => match create_args {
            CreateArgs::Form(filename) => {
                match &filename.name {
                    Some(name) => println!("Creating form from template: {:?}", name),
                    None => println!("Creating form from template: template.yaml")
                }
            }
        },
    }
}
