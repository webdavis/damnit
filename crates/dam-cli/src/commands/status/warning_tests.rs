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
fn status_does_not_warn_when_a_credential_is_a_value_in_the_config_file() {
    let mut ctx = context();
    ctx.config.remotes = vec![remote_with(vec![CredentialSpec::Literal {
        name: "api_token".into(),
        value: "SUPERSECRETTOKEN".into(),
    }])];
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    assert!(!report.human.contains("warning:"), "{}", report.human);
    assert_eq!(report.data["warnings"], serde_json::json!([]));
}
