use clap::{Args, Subcommand};

use formhandler::{ parse_yaml_file};
use std::path::PathBuf;
mod formhandler;

#[derive(Subcommand)]
pub enum CreateCommands {
    /// Adds files to myapp
    Form(Filename),
}

#[derive(Args)]
pub struct Filename {
    ///chooses the template yaml file for creating forms
    #[arg(value_name = "FILENAME")]
    name: PathBuf,
}

pub fn handle_create_form(filename: &Filename) -> Result<(), String> {
    if !filename.name.is_file() {
        return Err(format!(
            "Not a valid file: {:?}",
            filename.name.display()
        ));
    }

    println!("creating form with template : {:?}", filename.name);
    let form_template = parse_yaml_file(&filename.name)?;
    println!(
        "{:?}, {:?},{:?}",
        form_template.title, form_template.description, form_template.questions
    );

    Ok(())
}
