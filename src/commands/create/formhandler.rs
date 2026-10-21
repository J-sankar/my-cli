use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
};

use crate::commands::auth::google::GoogleRefreshResponse;

#[derive(serde::Deserialize)]
pub struct CreatedForm {
    #[serde(rename = "formId")]
    form_id: String,

    #[serde(rename = "responderUri")]
    pub responder_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FormData {
    pub title: String,
    pub description: Option<String>,
    pub questions: Vec<QuestionSpec>,
}

impl FormData {
    pub fn print_preview(&self) {
        println!("Dry run — no Google Form will be created.\n");

        println!("Title: {}", self.title);

        if let Some(description) = &self.description {
            println!("Description: {}", description.trim());
        }

        println!("Questions: {}\n", self.questions.len());

        for (index, question) in self.questions.iter().enumerate() {
            let required = if question.required {
                "required"
            } else {
                "optional"
            };

            println!(
                "{}. {} — {:?} — {}",
                index + 1,
                question.title,
                question.question_type,
                required
            );
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestionSpec {
    title: String,
    #[serde(rename = "type")]
    question_type: QuestionType,
    required: bool,
    options: Option<Vec<String>>,
}

impl QuestionSpec {
    fn get_type_as_string(&self) -> String {
        match &self.question_type {
            QuestionType::Checkboxes => "checkbox".to_string(),
            QuestionType::Dropdown => "dropdown".to_string(),
            QuestionType::Paragraph => "paragraph".to_string(),
            QuestionType::Radio => "radio".to_string(),
            QuestionType::ShortText => "shorttext".to_string(),
        }
    }

    fn validate_question(&self) -> Result<(), String> {
        let mut errors: String = String::new();
        if self.title.trim().is_empty() {
            errors.push_str("Question title cannot be empty\n");
        }

        match &self.question_type {
            QuestionType::Radio | QuestionType::Checkboxes | QuestionType::Dropdown => {
                match self.options.as_ref() {
                    Some(options) if options.len() >= 2 => {
                        let empty_options_count: usize = options
                            .iter()
                            .filter(|option| option.trim().is_empty())
                            .count();
                        if empty_options_count > 0 {
                            errors.push_str(&format!(
                                "Found {:?} blank options of type  {:?}\n",
                                empty_options_count,
                                self.get_type_as_string()
                            ));
                        }
                        for (option, count) in find_duplicate_options(options) {
                            if count > 1 {
                                errors.push_str(&format!(
                                    "Option '{}' appears {} times.\n",
                                    option, count
                                ));
                            }
                        }
                    }
                    _ => errors.push_str(
                        "Radio, checkbox, and dropdown questions need at least two options.\n",
                    ),
                }
            }
            QuestionType::Paragraph | QuestionType::ShortText => {
                if let Some(options) = &self.options
                    && !options.is_empty()
                {
                    errors.push_str("Short-text and paragraph questions cannot have options.\n")
                }
            }
        }
        if !errors.trim().is_empty() {
            return Err(errors);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum QuestionType {
    ShortText,
    Paragraph,
    Radio,
    Checkboxes,
    Dropdown,
}

fn find_duplicate_options(options: &[String]) -> HashMap<&str, usize> {
    let mut option_map: HashMap<&str, usize> = HashMap::new();
    for option in options {
        let count = option_map.entry(option.trim()).or_insert(0);
        *count += 1;
    }
    option_map
}

pub fn parse_yaml_file(file: &PathBuf) -> Result<FormData, String> {
    let file = fs::File::open(file).map_err(|error| error.to_string())?;
    let data: FormData = serde_yaml::from_reader(&file).map_err(|error| error.to_string())?;
    Ok(data)
}

pub fn validate_form_data(form_data: &FormData) -> Result<(), String> {
    println!("Validating form");
    let mut errors = String::new();
    if form_data.title.trim().is_empty() {
        errors.push_str("title empty in template file.\n");
    }

    if form_data.questions.is_empty() {
        errors.push_str("the template should contain atleast one question\n");
    }
    for (index, question) in form_data.questions.iter().enumerate() {
        if let Err(question_error) = question.validate_question() {
            errors.push_str(&format!(
                "Question {}\n{}\n",
                index + 1,
                question_error.trim()
            ));
        }
    }

    if !errors.trim().is_empty() {
        errors.insert_str(0, "Form Error\n");
        return Err(errors);
    }
    println!("No errors found");

    Ok(())
}

pub fn create_item(question: &QuestionSpec, index: usize) -> Result<serde_json::Value, String> {
    match &question.question_type {
        QuestionType::ShortText | QuestionType::Paragraph => {
            let question_type = "textQuestion".to_string();
            let is_paragraph = matches!(question.question_type, QuestionType::Paragraph);
            let item = serde_json::json!({"createItem":{"item":{"title": &question.title,"question_item":{"question":{"required":question.required,question_type:{"paragraph":is_paragraph}}} },"location":{"index":index}}});
            Ok(item)
        }

        QuestionType::Checkboxes | QuestionType::Dropdown | QuestionType::Radio => {
            let question_type = match question.question_type {
                QuestionType::Checkboxes => "CHECKBOX",
                QuestionType::Dropdown => "DROP_DOWN",
                QuestionType::Radio => "RADIO",
                _ => "CHOICE_TYPE_UNSPECIFIED",
            };
            let options: Vec<serde_json::Value> = question
                .options
                .as_deref() // Option<Vec<String>> -> Option<&[String]>
                .unwrap_or(&[])
                .iter()
                .map(|option| serde_json::json!({ "value": option }))
                .collect();

            let item = serde_json::json!({"createItem":{"item":{"title": &question.title,"question_item":{"question":{"required":question.required,"choiceQuestion": {"type":question_type, "options":options}}} },"location":{"index":index}}});
            Ok(item)
        }
    }
}
pub fn create_empty_form(
    title: &str,
    token_data: &GoogleRefreshResponse,
    http_client: &reqwest::blocking::Client,
) -> Result<serde_json::Value, String> {
    const CREATION_URI: &str = "https://forms.googleapis.com/v1/forms";
    let body = serde_json::json!({
        "info": {
            "title": title,
            "document_title":title
        }
    });
    let res = http_client
        .post(CREATION_URI)
        .bearer_auth(token_data.get_access_token())
        .json(&body)
        .send()
        .map_err(|err| err.to_string())?;
    if !res.status().is_success() {
        let err_body = res.text().unwrap_or_default();
        return Err(err_body.to_string());
    }
    let res_body: serde_json::Value = res.json().map_err(|err| err.to_string())?;

    Ok(res_body)
}

pub fn batch_update(
    form_items: &[serde_json::Value],
    created_form: &CreatedForm,
    token_data: &GoogleRefreshResponse,
    http_client: &reqwest::blocking::Client,
) -> Result<serde_json::Value, String> {
    let updation_uri: String = format!(
        "https://forms.googleapis.com/v1/forms/{}:batchUpdate",
        created_form.form_id
    );
    let body = serde_json::json!({
        "includeFormInResponse":true,
        "requests":
            &form_items

    });
    let res = http_client
        .post(updation_uri)
        .bearer_auth(token_data.get_access_token())
        .json(&body)
        .send()
        .map_err(|err| err.to_string())?;
    if !res.status().is_success() {
        let err_body = res.text().unwrap_or_default();
        return Err(err_body.to_string());
    }
    let res_body: serde_json::Value = res.json().map_err(|err| err.to_string())?;

    Ok(res_body)
}

pub fn update_form_info(
    title: &str,
    description: &str,
    token_data: &GoogleRefreshResponse,
    http_client: &reqwest::blocking::Client,
    created_form: &CreatedForm,
) -> Result<serde_json::Value, String> {
    let updation_uri: String = format!(
        "https://forms.googleapis.com/v1/forms/{}:batchUpdate",
        created_form.form_id
    );
    let body = serde_json::json!({
        "includeFormInResponse":true,
        "requests":
            [
                {"updateFormInfo": {
                    "info": {
                        "title":title,
                        "documentTitle":title,
                        "description":description
                    },
                    "updateMask": "description"
                }
            }
            ]

    });
    let res = http_client
        .post(updation_uri)
        .bearer_auth(token_data.get_access_token())
        .json(&body)
        .send()
        .map_err(|err| err.to_string())?;
    if !res.status().is_success() {
        let err_body = res.text().unwrap_or_default();
        return Err(err_body.to_string());
    }
    let res_body: serde_json::Value = res.json().map_err(|err| err.to_string())?;
    // println!("{}",res_body);
    Ok(res_body)
}
