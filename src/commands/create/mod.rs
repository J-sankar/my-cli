use clap::{Args, Subcommand};

use formhandler::{ parse_yaml_file,validate_form_data};
use std::path::PathBuf;
mod formhandler;

#[derive(Subcommand)]
pub enum CreateCommands {
    /// create a google for from a yaml template
    Form(Filename),
}

#[derive(Args)]
pub struct Filename {
    ///chooses the template yaml file for creating forms
    #[arg(value_name = "FILENAME")]
    name: PathBuf,

    /// Validate and preview the template without creating a form.
    #[arg(long)]
    dry_run: bool

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

    validate_form_data(&form_template)?;

    if filename.dry_run {
        form_template.print_preview();
        return  Ok(());
    }
    println!(
        "{:?}, {:?},{:?}",
        form_template.title, form_template.description, form_template.questions
    );
    Ok(())
}
