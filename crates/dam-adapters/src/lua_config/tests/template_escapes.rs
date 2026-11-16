use std::time::{Duration, Instant};

use dam_application::ConfigError;

use super::template_support::{Folders, found};

fn refused_naming_only_the_file(text: &str) -> Folders {
    let folders = Folders::new().project_template("bug", text);
    match found(&folders.inside(), "bug").unwrap_err() {
        ConfigError::StoppedLoading { path, reason, line } => {
            assert_eq!(path, folders.project_file("bug"), "{text}");
            assert_eq!((reason, line), (None, None), "{text}");
        }
        other => panic!("{text}: {other:?}"),
    }
    folders
}

#[test]
fn a_projects_template_cannot_reach_the_system() {
    for text in [
        "require('os') return {}",
        "return { body = string.rep('x', 3) }",
        "error('\\27[31mred') return {}",
        "return { subject = 'x' }",
        "return { priority = '\\27[31m' }",
    ] {
        refused_naming_only_the_file(text);
    }
}

#[test]
fn a_projects_template_runs_no_command_and_writes_no_file() {
    let marks = tempfile::tempdir().unwrap();
    let mark = marks.path().join("MARK").display().to_string();
    for text in [
        format!("os.execute('touch {mark}') return {{}}"),
        format!("io.open('{mark}', 'w') return {{}}"),
    ] {
        refused_naming_only_the_file(&text);
        assert!(!marks.path().join("MARK").exists(), "{text}");
    }
}

#[test]
fn an_endless_loop_in_a_projects_template_is_stopped() {
    let started = Instant::now();
    refused_naming_only_the_file("while true do end return {}");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn an_escape_sequence_in_a_projects_template_is_refused() {
    for text in [
        "return { body = '\\27[2J' }",
        "return { labels = { 'ok', '\\27]0;x\\7' } }",
        "return { due = 'tomorrow\\r' }",
        "return { path = 'a\\27/' }",
    ] {
        let folders = Folders::new().project_template("bug", text);
        assert_eq!(
            found(&folders.inside(), "bug").unwrap_err(),
            ConfigError::HoldsAControlCharacter {
                file: folders.project_file("bug")
            },
            "{text}"
        );
    }
}

#[test]
fn a_projects_template_body_may_hold_lines_and_tabs() {
    let folders = Folders::new().project_template("bug", "return { body = 'a\\n\\tb' }");
    let template = found(&folders.inside(), "bug").unwrap().unwrap();
    assert_eq!(template.body.as_deref(), Some("a\n\tb"));
}
