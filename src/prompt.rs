use super::cli::{Args, Format};

pub fn create_prompt(diff: &str, template: &String, args: &Args) -> String {
    let mut prompt = template.clone();
    if let Some(format) = &args.format {
        let additional_prompt = match format {
            Format::Formal => {
                "\n - Write a concise and formal Git commit message. Adhere to the Conventional Commits specification.
    
                Structure: <type>(<scope>): <description>
    
                Examples:
                - feat(authentication): Add user login feature
                - fix(deps): Update lodash to fix security vulnerability
                - docs: Update README with installation instructions
                - chore: Refactor build script
    
                Start with a type (feat, fix, docs, chore, style, refactor, test, build, ci, perf, revert) followed by an optional scope in parentheses, a colon, and a space. The description should be a short, imperative sentence. Use a blank line between the subject and body if a body is needed.\n"
            },
            Format::Casual => {
                 "\n - Write a brief and clear commit message that summarizes your changes. There are no strict format rules, but aim for readability.
    
                Examples:
                - Added login screen
                - Fixed a bug on the user profile page
                - Updated documentation
                - Small refactorings
    
                Just describe what you did in a straightforward way. Keep it short if possible.\n"
            }};
        prompt.push_str(additional_prompt);
    }

    if let Some(start) = &args.start {
        let additional_prompt = format!("\n - start with {}\n", start);
        prompt.push_str(&additional_prompt);
    }
    if let Some(include) = &args.include {
        let additional_prompt = format!(
            "\n - include the following words properly: {}\n",
            include.join(",")
        );
        prompt.push_str(&additional_prompt);
    }
    if let Some(lang) = &args.lang {
        let additional_prompt = format!(
            "\n - write the commit message in the following language: {}. If it's not a natural language, please write in English \n",
            &lang
        );
        prompt.push_str(&additional_prompt);
    }

    let diff_description = format!("--- \n Below are the staged modifications\n {diff}");
    prompt.push_str(&diff_description);
    prompt
}
