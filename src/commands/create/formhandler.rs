use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    path::PathBuf,
};


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

        if let Some(description)  = &self.description{
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
    fn validate_question(&self) -> Result<(), String> {
        let mut errors: String = String::new();
        if self.title.trim().is_empty() {
            errors.push_str("Question title cannot be empty\n");
        }

        match &self.question_type {
            QuestionType::Radio | QuestionType::Checkboxes | QuestionType::Dropdown => {
                match self.options.as_ref() {
                    Some(options) if options.len() >= 2 => {}
                    _ => errors.push_str(
                        "Radio, checkbox, and dropdown questions need at least two options.\n",
                    ),
                }
            }
            _ => {}
        }
        if !errors.trim().is_empty() {
            return Err(errors);
        }
        Ok(())
    }
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
    for (index,question) in form_data.questions.iter().enumerate() {
        if let Err(question_error) = question.validate_question() {
            errors.push_str(&format!("Question {}\n{}\n",
            index + 1,
            
            question_error.trim()));
        }
        errors.push_str("\n");
    }
    if !errors.trim().is_empty() {
        errors.insert_str(0, "Form Error\n");
        return Err(errors);
    }
    println!("No errors found");

    Ok(())
}
