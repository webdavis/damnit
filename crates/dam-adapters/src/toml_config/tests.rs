use super::*;
use dam_application::CredentialSpec;
use std::time::Duration;

const FULL: &str = r#"
[done]
interactive = true

[remote.todoist]
url = "todoist::"
api_token_command = ["security", "find-generic-password", "-w", "-s", "Todoist"]
stale = "15m"
path = "work/"

[remote.gcal]
url = "gcal::"
client_id = "abc"
refresh_token_env = "DAM_GCAL_REFRESH"

[category.effort]
values = ["light", "deep"]
exclusive = true

[filter.today]
query = "due:today | overdue"
"#;

#[test]
fn a_full_file_parses_into_config() {
    let c = parse_config(FULL).unwrap();
    assert!(c.done_interactive);
    let t = c.remote("todoist").unwrap();
    assert_eq!(t.helper, "todoist");
    assert_eq!(t.stale, Some(Duration::from_secs(900)));
    assert_eq!(t.path.as_ref().unwrap().as_str(), "work/");
    assert_eq!(
        t.credentials,
        vec![CredentialSpec::Command {
            name: "api_token".into(),
            argv: vec![
                "security".into(),
                "find-generic-password".into(),
                "-w".into(),
                "-s".into(),
                "Todoist".into()
            ]
        }]
    );
    let g = c.remote("gcal").unwrap();
    assert!(g.credentials.contains(&CredentialSpec::Literal {
        name: "client_id".into(),
        value: "abc".into()
    }));
    assert!(g.credentials.contains(&CredentialSpec::Env {
        name: "refresh_token".into(),
        var: "DAM_GCAL_REFRESH".into()
    }));
    assert!(c.categories.category_of("deep").is_some());
    assert_eq!(c.filter("today").unwrap().query, "due:today | overdue");
}

#[test]
fn an_empty_file_is_the_default() {
    let c = parse_config("").unwrap();
    assert!(!c.done_interactive);
    assert!(c.remotes.is_empty());
    assert!(c.filters.is_empty());
}

#[test]
fn a_remote_without_a_helper_prefix_is_invalid() {
    let err = parse_config("[remote.x]\nurl = \"nope\"\n").unwrap_err();
    assert!(matches!(err, ConfigError::Invalid(m) if m.contains("remote.x")));
}

#[test]
fn a_command_credential_must_be_an_array() {
    let err = parse_config("[remote.x]\nurl = \"t::\"\napi_token_command = \"a b\"\n").unwrap_err();
    assert!(matches!(err, ConfigError::Invalid(m) if m.contains("api_token_command")));
}

#[test]
fn stale_accepts_seconds_minutes_and_hours_only() {
    assert_eq!(parse_stale("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_stale("15m").unwrap(), Duration::from_secs(900));
    assert_eq!(parse_stale("2h").unwrap(), Duration::from_secs(7200));
    assert!(parse_stale("15").is_err());
    assert!(parse_stale("1d").is_err());
}

#[test]
fn append_remote_adds_a_table_and_refuses_a_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[done]\ninteractive = true").unwrap();
    append_remote(&path, "todoist", "todoist::").unwrap();
    let c = load_config(&path).unwrap();
    assert_eq!(c.remote("todoist").unwrap().helper, "todoist");
    assert!(c.done_interactive);
    assert!(matches!(
        append_remote(&path, "todoist", "todoist::"),
        Err(ConfigError::Invalid(_))
    ));
}

#[test]
fn a_missing_file_loads_as_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let c = load_config(&dir.path().join("absent.toml")).unwrap();
    assert!(c.remotes.is_empty());
}
