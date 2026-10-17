use serde::{Deserialize, Serialize};
use std::{fs::{self}, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct FormData {
    pub title: String,
    pub description: String,
    pub questions: Vec<QuestionSpec>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestionSpec {
    title: String,
    #[serde(rename = "type")]
    question_type: QuestionType,
    required: bool,
    options: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum QuestionType {
    ShortText,
    Paragraph,
    Radio,
    Checkboxes,
    Dropdown,
}


pub fn parse_yaml_file(file:&PathBuf) -> Result<FormData,String> {
    let file = fs::File::open(file).map_err(|error| error.to_string())?;
    let data: FormData = serde_yaml::from_reader(&file).map_err(|error| error.to_string())?;
    Ok(data)
}