use clap::{Args, Subcommand};
use serde_json::Value;

use crate::commands::auth::google::{self, GoogleRefreshResponse, Unauthorized};
use crate::commands::create::formhandler::{create_item, update_form_info};
use crate::utils::load_token;
use formhandler::{parse_yaml_file, validate_form_data};

use std::path::PathBuf;
mod formhandler;

use formhandler::{CreatedForm, batch_update, create_empty_form};

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
    dry_run: bool,
}

pub fn handle_create_form(filename: &Filename) -> Result<(), String> {
    if !filename.name.is_file() {
        return Err(format!("Not a valid file: {:?}", filename.name.display()));
    }

    println!("creating form with template : {:?}", filename.name);
    let form_template = parse_yaml_file(&filename.name)?;

    validate_form_data(&form_template)?;

    if filename.dry_run {
        form_template.print_preview();
        return Ok(());
    }
    let refresh_token = match load_token() {
        Ok(token) => token,
        Err(e) => {
            if let Some(keyring::Error::NoEntry) = e.downcast_ref::<keyring::Error>() {
                return Err("you have been logged out, please run login".to_string());
            } else {
                return Err(format!("{e}"));
            }
        }
    };
    let token_response = match google::get_new_token(&refresh_token) {
        Ok(token_res) => token_res,
        Err(e) => {
            if e.is::<Unauthorized>() {
                return Err("you have been logged out, please run login".to_string());
            } else {
                return Err(format!("{e}"));
            }
        }
    };
    let http_client = reqwest::blocking::Client::new();
    let res = create_empty_form(&form_template.title, &token_response, &http_client)?;
    let form_items: Vec<Value> = form_template
        .questions
        .iter()
        .enumerate()
        .map(|(index, question)| create_item(question, index))
        .collect::<Result<_, _>>()?;

    let created_form: CreatedForm = serde_json::from_value(res).map_err(|err| err.to_string())?;
    let description = form_template.description.as_deref().unwrap_or_default();
    let update_info = update_form_info(
        &form_template.title,
        description,
        &token_response,
        &http_client,
        &created_form,
    )?;
    let edition_res = batch_update(&form_items, &created_form, &token_response, &http_client)?;
    println!(
        "form created successfully:\nResponsders' Url : {}",
        created_form.responder_uri
    );

    Ok(())
}
