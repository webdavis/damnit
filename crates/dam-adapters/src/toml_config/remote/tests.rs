use crate::toml_config::parse_config;
use dam_application::ConfigError;
use dam_application::CredentialSpec;
use std::time::Duration;

#[test]
fn a_remote_without_a_helper_prefix_is_invalid() {
    let err = parse_config("[remote.x]\nurl = \"nope\"\n").unwrap_err();
    assert!(matches!(err, ConfigError::Invalid(m) if m.contains("remote.x")));
}

#[test]
fn a_hand_written_remote_name_outside_the_bare_key_set_is_refused_by_name() {
    let err = parse_config("[remote.\"a.b\"]\nurl = \"t::\"\n").unwrap_err();
    assert_eq!(
        err.to_string(),
        "config: remote name \"a.b\" may hold only letters, digits, - and _"
    );
}

#[test]
fn a_command_credential_must_be_an_array() {
    let err = parse_config("[remote.x]\nurl = \"t::\"\napi_token_command = \"a b\"\n").unwrap_err();
    assert!(matches!(err, ConfigError::Invalid(m) if m.contains("api_token_command")));
}

#[test]
fn an_absent_deadline_is_none_and_a_non_string_is_refused() {
    let c = parse_config("[remote.x]\nurl = \"t::\"\n").unwrap();
    assert_eq!(c.remote("x").unwrap().deadline, None);
    let err = parse_config("[remote.x]\nurl = \"t::\"\ndeadline = 60\n").unwrap_err();
    assert!(matches!(err, ConfigError::Invalid(m) if m.contains("deadline must be a string")));
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
