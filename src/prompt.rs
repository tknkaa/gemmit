use super::cli::Args;
use std::fs;

pub fn create_prompt(diff: &str, filepath: &str, args: &Args) -> String {
    let mut template = fs::read_to_string(filepath).unwrap();
    if let Some(start) = &args.start {
        let additional_prompt = format!("\n start with {}\n", start);
        template.push_str(&additional_prompt);
    }
    if let Some(include) = &args.include {
        let additional_prompt = format!(
            "\n include the following words properly: {}\n",
            include.join(",")
        );
        template.push_str(&additional_prompt);
    }
    if let Some(lang) = &args.lang {
        let additional_prompt = format!(
            "\n write the commit message in the following language: {}. if it's not a natural language, please write in English \n",
            &lang
        );
        template.push_str(&additional_prompt);
    }
    template.push_str(diff);
    template
}
