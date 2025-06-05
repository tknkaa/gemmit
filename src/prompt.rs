use std::fs;

pub fn create_prompt(diff: &str, filepath: &str) -> String {
    let mut template = fs::read_to_string(filepath).unwrap();
    template.push_str(diff);
    template
}
