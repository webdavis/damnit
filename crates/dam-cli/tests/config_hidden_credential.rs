mod support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn with_credentials(remotes: &str, credentials: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!("return {{ remote = {{ {remotes} }} }}\n"));
    sb.write_secrets_lua(&format!(
        "return {{ schema = 1, remote = {{ {credentials} }} }}\n"
    ));
    sb
}

fn listed(sb: &Sandbox) -> Value {
    let out = sb.output(&["config", "list", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let answer = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-list.schema.json");
    answer
}

#[test]
fn a_credential_is_hidden_and_says_whether_dam_calls_it() {
    let _guard = support::guard("a_credential_is_hidden_and_says_whether_dam_calls_it");
    let sb = with_credentials(
        "todoist = { url = 'todoist::' }",
        "todoist = { api_token = 'abc', refresh = function() return 'x' end }",
    );
    let credential = &listed(&sb)["settings"]["remote"]["todoist"]["credential"];
    assert_eq!(
        credential,
        &json!({
            "api_token": {"hidden": true, "function": false},
            "refresh": {"hidden": true, "function": true},
        })
    );
    let (ok, out, err) = sb.dam(&["config", "get", "remote.todoist.credential"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "  remote.todoist.credential.api_token  hidden\n  remote.todoist.credential.refresh    hidden (function)\n"
    );
}

#[test]
fn parts_named_like_the_fields_of_a_setting_still_match_the_schema() {
    let _guard = support::guard("parts_named_like_the_fields_of_a_setting_still_match_the_schema");
    let names = ["hidden", "function", "value", "default"];
    let remotes: Vec<String> = names
        .iter()
        .map(|n| format!("['{n}'] = {{ url = 'a::' }}"))
        .collect();
    let credentials: Vec<String> = names
        .iter()
        .map(|n| format!("['{n}'] = {{ hidden = 'abc', ['function'] = 'abc' }}"))
        .collect();
    let sb = with_credentials(&remotes.join(", "), &credentials.join(", "));
    let answer = listed(&sb);
    assert_eq!(
        answer["settings"]["remote"]["hidden"]["credential"]["function"],
        json!({"hidden": true, "function": false})
    );
}
