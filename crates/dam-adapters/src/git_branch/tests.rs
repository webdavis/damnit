use std::fs;

use super::checked_out_branch;

fn repo_with_head(head: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".git")).unwrap();
    fs::write(root.path().join(".git/HEAD"), head).unwrap();
    root
}

#[test]
fn the_branch_is_read_from_head() {
    let root = repo_with_head("ref: refs/heads/fix/login\n");
    assert_eq!(
        checked_out_branch(root.path()).as_deref(),
        Some("fix/login")
    );
}

#[test]
fn a_folder_below_the_repository_finds_its_head() {
    let root = repo_with_head("ref: refs/heads/main\n");
    let deep = root.path().join("app/cli");
    fs::create_dir_all(&deep).unwrap();
    assert_eq!(checked_out_branch(&deep).as_deref(), Some("main"));
}

#[test]
fn a_detached_head_or_no_repository_is_no_branch() {
    let detached = repo_with_head("3f786850e387550fdab836ed7e6dc881de23001b\n");
    assert_eq!(checked_out_branch(detached.path()), None);
    let bare = tempfile::tempdir().unwrap();
    assert_eq!(checked_out_branch(bare.path()), None);
}

#[test]
fn a_git_file_is_followed_to_its_gitdir() {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("main-repo/.git/worktrees/wt");
    fs::create_dir_all(&real).unwrap();
    fs::write(real.join("HEAD"), "ref: refs/heads/feature\n").unwrap();
    let worktree = root.path().join("wt");
    fs::create_dir_all(&worktree).unwrap();
    fs::write(
        worktree.join(".git"),
        "gitdir: ../main-repo/.git/worktrees/wt\n",
    )
    .unwrap();
    assert_eq!(checked_out_branch(&worktree).as_deref(), Some("feature"));
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", real.display()),
    )
    .unwrap();
    assert_eq!(checked_out_branch(&worktree).as_deref(), Some("feature"));
}

#[test]
fn a_head_too_large_to_be_one_is_no_branch() {
    let long = format!("ref: refs/heads/{}\n", "a".repeat(10_000));
    let root = repo_with_head(&long);
    assert_eq!(checked_out_branch(root.path()), None);
}
