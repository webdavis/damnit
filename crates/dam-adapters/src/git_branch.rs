use std::io::Read;
use std::path::{Path, PathBuf};

const LARGEST_HEAD: u64 = 1024;

pub fn checked_out_branch(folder: &Path) -> Option<String> {
    let dot_git = folder
        .ancestors()
        .map(|f| f.join(".git"))
        .find(|g| g.exists())?;
    let git_dir = match dot_git.is_file() {
        true => followed(&dot_git)?,
        false => dot_git,
    };
    let head = small_text(&git_dir.join("HEAD"))?;
    head.trim_end()
        .strip_prefix("ref: refs/heads/")
        .map(String::from)
}

fn followed(dot_git: &Path) -> Option<PathBuf> {
    let text = small_text(dot_git)?;
    let named = text.trim_end().strip_prefix("gitdir: ")?;
    Some(dot_git.parent()?.join(named))
}

fn small_text(file: &Path) -> Option<String> {
    let mut text = String::new();
    std::fs::File::open(file)
        .ok()?
        .take(LARGEST_HEAD + 1)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() as u64 <= LARGEST_HEAD).then_some(text)
}

#[cfg(test)]
mod tests;
