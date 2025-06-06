use super::cli::Args;
use std::fs;

pub fn create_prompt(diff: &str, filepath: &str, args: &Args) -> String {
    let mut template = fs::read_to_string(filepath).unwrap();
    if let Some(start) = &args.start {
        let additional_prompt = format!("\nstart with {}\n", start);
        template.push_str(&additional_prompt);
    }
    template.push_str(diff);
    template
}
