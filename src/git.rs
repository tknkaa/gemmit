use std::process::Command;

const WARNING_DIRS: &[&str] = &["node_modules/", ".direnv/"];
const LOCK_EXCLUDES: &[&str] = &[
    ":(exclude)go.sum",
    ":(exclude)go.mod",
    ":(exclude)package-lock.json",
    ":(exclude)yarn.lock",
    ":(exclude)pnpm-lock.yaml",
    ":(exclude)bun.lock",
    ":(exclude)Cargo.lock",
    ":(exclude)poetry.lock",
    ":(exclude)uv.lock",
    ":(exclude)Gemfile.lock",
    ":(exclude)flake.lock",
];

pub enum BgMsg {
    /// Git info gathered; prompt is ready, caller decides what to do next.
    Prepared {
        prompt: String,
        warning_dirs: Vec<String>,
    },
    /// Gemini returned a commit message.
    Generated(String),
    GenerateErr(String),
    CommitDone,
    CommitErr(String),
}

pub fn git_prepare() -> BgMsg {
    let staged = match Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => return BgMsg::GenerateErr(format!("Failed to run git: {e}")),
    };

    if staged.trim().is_empty() {
        return BgMsg::GenerateErr("no changes are staged".to_string());
    }

    let mut diff_args = vec!["diff", "--cached", "--"];
    diff_args.extend_from_slice(LOCK_EXCLUDES);

    let diff = match Command::new("git").args(&diff_args).output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => return BgMsg::GenerateErr(format!("Failed to get git diff: {e}")),
    };

    let prompt = format!(
        "Generate a concise conventional commit message (feat/fix/chore prefix) for this diff. \
         Keep it short and to the point - ideally one line. \
         Return only the commit message, no explanations or formatting:\n\n{diff}"
    );

    let warning_dirs: Vec<String> = WARNING_DIRS
        .iter()
        .filter(|&&d| staged.contains(d))
        .map(|&s| s.to_string())
        .collect();

    BgMsg::Prepared {
        prompt,
        warning_dirs,
    }
}

pub fn git_commit(message: &str) -> BgMsg {
    let result = Command::new("git")
        .args(["commit", "-m", message])
        .output();
    match result {
        Ok(o) if o.status.success() => BgMsg::CommitDone,
        Ok(o) => BgMsg::CommitErr(String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => BgMsg::CommitErr(e.to_string()),
    }
}
