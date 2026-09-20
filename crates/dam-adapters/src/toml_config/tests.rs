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
deadline = "2m"
path = "work/"

[remote.gcal]
url = "gcal::"
credentials = ["client_id"]
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
    assert_eq!(
        t.deadline,
        Some(dam_application::ConfiguredDuration {
            value: Duration::from_secs(120),
            text: "2m".into()
        })
    );
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
    assert_eq!(
        parse_duration("stale", "30s").unwrap(),
        Duration::from_secs(30)
    );
    assert_eq!(
        parse_duration("stale", "15m").unwrap(),
        Duration::from_secs(900)
    );
    assert_eq!(
        parse_duration("stale", "2h").unwrap(),
        Duration::from_secs(7200)
    );
    assert!(parse_duration("stale", "15").is_err());
    assert!(parse_duration("stale", "1d").is_err());
}

#[test]
fn a_non_ascii_or_empty_duration_is_refused_rather_than_a_panic() {
    assert!(parse_duration("stale", "15\u{00b5}").is_err());
    assert!(parse_duration("stale", "").is_err());
    assert!(parse_duration("deadline", "\u{201c}2m\u{201d}").is_err());
    assert!(parse_duration("stale", "18446744073709551615h").is_err());
}

#[test]
fn an_absent_deadline_is_none_and_a_non_string_is_refused() {
    let c = parse_config("[remote.x]\nurl = \"t::\"\n").unwrap();
    assert_eq!(c.remote("x").unwrap().deadline, None);
    let err = parse_config("[remote.x]\nurl = \"t::\"\ndeadline = 60\n").unwrap_err();
    assert!(matches!(err, ConfigError::Invalid(m) if m.contains("deadline must be a string")));
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

#[test]
fn a_syntax_error_names_its_position_and_never_echoes_the_file() {
    const TOKEN: &str = "SUPERSECRETTOKEN";
    let on_the_token_line =
        format!("[remote.todoist]\nurl = \"todoist::\"\napi_token = \"{TOKEN}\n");
    let two_lines_away =
        format!("[remote.todoist]\napi_token = \"{TOKEN}\"\nurl = \"todoist::\"\nstale = 15m\n");
    for text in [on_the_token_line, two_lines_away] {
        let rendered = parse_config(&text).unwrap_err().to_string();
        assert!(!rendered.contains(TOKEN), "{rendered}");
        assert!(
            rendered.contains("line 3") || rendered.contains("line 4"),
            "{rendered}"
        );
    }
}

fn mode_of(path: &FsPath) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn append_remote_creates_a_private_file_in_a_private_directory() {
    let base = tempfile::tempdir().unwrap();
    let dir = base.path().join("dam");
    let path = dir.join("config.toml");
    append_remote(&path, "todoist", "todoist::").unwrap();
    assert_eq!(mode_of(&path), 0o600);
    assert_eq!(mode_of(&dir), 0o700);
}

#[test]
fn append_remote_tightens_a_world_readable_config() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[done]\ninteractive = true\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    append_remote(&path, "todoist", "todoist::").unwrap();
    assert_eq!(mode_of(&path), 0o600);
    assert!(load_config(&path).unwrap().done_interactive);
}

#[test]
fn a_misspelled_remote_key_is_refused_by_name_and_never_becomes_a_credential() {
    let err = parse_config("[remote.x]\nurl = \"t::\"\nstael = \"15m\"\n").unwrap_err();
    assert!(
        matches!(&err, ConfigError::Invalid(m) if m.contains("remote.x") && m.contains("stael")),
        "{err}"
    );
    let good = parse_config("[remote.x]\nurl = \"t::\"\nstale = \"15m\"\n").unwrap();
    assert_eq!(
        good.remote("x").unwrap().stale,
        Some(Duration::from_secs(900))
    );
}

#[test]
fn a_literal_credential_is_read_only_when_credentials_declares_its_name() {
    let text = "[remote.x]\nurl = \"t::\"\ncredentials = [\"api_token\"]\napi_token = \"tok\"\n";
    assert_eq!(
        parse_config(text).unwrap().remote("x").unwrap().credentials,
        vec![CredentialSpec::Literal {
            name: "api_token".into(),
            value: "tok".into()
        }]
    );
    let undeclared = parse_config("[remote.x]\nurl = \"t::\"\napi_token = \"tok\"\n").unwrap_err();
    assert!(
        matches!(&undeclared, ConfigError::Invalid(m) if m.contains("api_token")),
        "{undeclared}"
    );
}

#[test]
fn the_config_value_wins_over_the_command_and_the_command_over_the_variable() {
    let text = r#"
[remote.t]
url = "t::"
credentials = ["api_token"]
api_token_env = "T_TOKEN"
api_token_command = ["print-it"]
api_token = "the-value"
"#;
    let config = parse_config(text).unwrap();
    let remote = config.remote("t").unwrap();
    assert_eq!(remote.credentials.len(), 1, "one spec per credential name");
    assert!(
        matches!(&remote.credentials[0], CredentialSpec::Literal { .. }),
        "{:?}",
        remote.credentials[0]
    );
}

#[test]
fn the_command_wins_over_the_variable_when_no_value_is_set() {
    let text = r#"
[remote.t]
url = "t::"
api_token_env = "T_TOKEN"
api_token_command = ["print-it"]
"#;
    let config = parse_config(text).unwrap();
    let remote = config.remote("t").unwrap();
    assert_eq!(remote.credentials.len(), 1);
    assert!(
        matches!(&remote.credentials[0], CredentialSpec::Command { .. }),
        "{:?}",
        remote.credentials[0]
    );
}
