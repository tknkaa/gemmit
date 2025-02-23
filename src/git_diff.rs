use git2::{DiffFormat, Repository};

pub fn get_git_diff() -> Result<String, git2::Error> {
    let mut prompt_diff = String::new();

    let repo = Repository::open(".")?;
    let head_commit = repo.head()?.peel_to_commit()?;
    let head_tree = head_commit.tree()?;
    let index = repo.index()?;

    let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), None)?;

    diff.print(DiffFormat::Patch, |_, _, line| {
        let diff_line = format!(
            "{} {}",
            line.origin(),
            std::str::from_utf8(line.content()).unwrap()
        );
        prompt_diff.push_str(&diff_line);
        true
    })?;

    Ok(prompt_diff)
}
