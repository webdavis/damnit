use std::path::Path as FsPath;

use dam_domain::Path;
use tempfile::TempDir;

use super::Standing;
use crate::args::ScopeArgs;
use crate::error::CliError;

fn folder_holding(file: Option<&str>) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("app/src")).unwrap();
    if let Some(text) = file {
        std::fs::write(root.path().join("app/.dam.lua"), text).unwrap();
    }
    root
}

fn standing_in(root: &TempDir) -> Standing {
    Standing::of(&root.path().join("app/src"), FsPath::new("/nowhere"))
}

fn global() -> ScopeArgs {
    ScopeArgs {
        global: true,
        ..ScopeArgs::default()
    }
}

#[test]
fn inside_a_project_the_place_is_the_projects_path() {
    let root = folder_holding(Some(r#"return { path = "damnit" }"#));
    assert_eq!(
        standing_in(&root).place(&ScopeArgs::default()).unwrap(),
        Path::parse("damnit/").unwrap()
    );
}

#[test]
fn outside_any_project_and_with_global_the_place_is_the_root() {
    let outside = folder_holding(None);
    assert_eq!(
        standing_in(&outside).place(&ScopeArgs::default()).unwrap(),
        Path::root()
    );
    let inside = folder_holding(Some(r#"return { path = "damnit" }"#));
    assert_eq!(standing_in(&inside).place(&global()).unwrap(), Path::root());
}

#[test]
fn a_project_file_that_fails_to_load_is_refused_without_quoting_it() {
    let root = folder_holding(Some("return { path = 'secret-text' "));
    let err = standing_in(&root).place(&ScopeArgs::default()).unwrap_err();
    assert!(matches!(err, CliError::Config(_)), "{err:?}");
    assert!(!err.to_string().contains("secret-text"), "{err}");
}

#[test]
fn global_does_not_read_a_project_file_that_fails_to_load() {
    let root = folder_holding(Some("return {"));
    assert_eq!(standing_in(&root).place(&global()).unwrap(), Path::root());
}
#[test]
fn the_personal_config_folder_is_never_a_project_whatever_form_its_path_takes() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(config.join(".dam.lua"), r#"return { path = "x" }"#).unwrap();
    let linked = root.path().join("linked");
    std::os::unix::fs::symlink(&config, &linked).unwrap();
    let canonical = std::fs::canonicalize(&config).unwrap();
    let run = super::RunFolders::new(
        canonical,
        &linked.join("config.lua"),
        &root.path().join("dam.db"),
    );
    let standing = Standing::of(&run.folder, &run.personal_config);
    assert!(matches!(standing, Standing::Outside), "{standing:?}");
}

#[test]
fn a_project_inside_another_uses_only_its_own_file() {
    let root = folder_holding(Some(r#"return { path = "inner" }"#));
    std::fs::write(root.path().join(".dam.lua"), r#"return { path = "outer" }"#).unwrap();
    assert_eq!(
        standing_in(&root).place(&ScopeArgs::default()).unwrap(),
        Path::parse("inner/").unwrap()
    );
}

#[test]
fn project_stands_in_for_the_project_from_any_folder() {
    let deep = Path::parse("pns/hooks/").unwrap();
    let scope = ScopeArgs {
        project: Some(deep.clone()),
        ..ScopeArgs::default()
    };
    let unreadable = folder_holding(Some("return {"));
    assert_eq!(standing_in(&unreadable).place(&scope).unwrap(), deep);
    assert_eq!(
        standing_in(&folder_holding(None)).place(&scope).unwrap(),
        deep
    );
}

#[test]
fn global_with_project_is_a_usage_conflict_on_every_scoped_command() {
    use clap::Parser;
    for command in ["new x", "ls", "agenda", "reminders"] {
        let mut line: Vec<&str> = vec!["dam"];
        line.extend(command.split(' '));
        line.extend(["--global", "--project", "pns/"]);
        let refused = crate::args::Cli::try_parse_from(&line).unwrap_err();
        assert_eq!(refused.exit_code(), 2, "{command}");
        assert_eq!(refused.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}

#[test]
fn an_unreadable_project_file_gives_one_warning_line_that_never_quotes_it() {
    let root = folder_holding(Some("return { path = 'secret-text' "));
    let warning = standing_in(&root).warning().unwrap();
    assert!(warning.starts_with("dam: warning: "), "{warning}");
    assert!(warning.ends_with("; run dam doctor config"), "{warning}");
    assert_eq!(warning.lines().count(), 1);
    assert!(!warning.contains("secret-text"), "{warning}");
    let readable = folder_holding(Some(r#"return { path = "damnit" }"#));
    assert_eq!(standing_in(&readable).warning(), None);
    assert_eq!(standing_in(&folder_holding(None)).warning(), None);
}

#[test]
fn the_warning_for_a_file_holding_an_escape_sequence_holds_no_control_character() {
    let root = folder_holding(Some(
        "return { [\"\\27[2J\"] = 1, path = \"\\27]0;evil\\7\" }",
    ));
    let warning = standing_in(&root).warning().unwrap();
    assert!(!warning.chars().any(char::is_control), "{warning:?}");
    assert!(
        !warning.contains("evil") && !warning.contains("[2J"),
        "{warning}"
    );
}
