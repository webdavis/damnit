use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{CredentialSpec, RemoteConfig, RemoteName};

fn remote_with(credentials: Vec<CredentialSpec>) -> RemoteConfig {
    RemoteConfig {
        name: RemoteName("todoist".into()),
        helper: "todoist".into(),
        url: "todoist::".into(),
        credentials,
        stale: None,
        deadline: None,
        path: None,
    }
}

#[test]
fn status_warns_once_when_a_credential_is_a_value_in_the_config_file() {
    let mut ctx = context();
    ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Literal {
        name: "api_token".into(),
        value: "SUPERSECRETTOKEN".into(),
    }])];
    let report = super::run_status(&mut ctx, StatusArgs { full: false }).unwrap();
    let warnings: Vec<&str> = report
        .human
        .lines()
        .filter(|l| l.starts_with("warning:"))
        .collect();
    assert_eq!(warnings.len(), 1, "{}", report.human);
    assert!(warnings[0].contains("todoist.api_token"), "{}", warnings[0]);
    assert!(!report.human.contains("SUPERSECRETTOKEN"));
    assert_eq!(report.data["warnings"], serde_json::json!([warnings[0]]));
}

#[test]
fn status_does_not_warn_when_no_credential_is_a_value_in_the_config_file() {
    let mut ctx = context();
    ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Env {
        name: "api_token".into(),
        var: "DAM_TODOIST_API_TOKEN".into(),
    }])];
    let report = super::run_status(&mut ctx, StatusArgs { full: false }).unwrap();
    assert!(!report.human.contains("warning:"), "{}", report.human);
    assert_eq!(report.data["warnings"], serde_json::json!([]));
}
