use dam_application::testing::task_caps;
use dam_application::{CredentialSpec, HelperError, RemoteConfig, RemoteName};
use serde_json::{Value, json};

use super::checked;
use crate::commands::doctor::machine::Answer;

const TRY: &str = "run dam test credentials";
const SET_IT: &str = "set api_token under remote.fake in secrets.lua";

fn literal(value: &str) -> CredentialSpec {
    CredentialSpec::Literal {
        name: "api_token".into(),
        value: value.into(),
    }
}

fn answer(credentials: Vec<CredentialSpec>, asked_for: &[&str]) -> Answer {
    let mut capabilities = task_caps();
    capabilities.credentials = asked_for.iter().map(|name| name.to_string()).collect();
    Answer {
        remote: RemoteConfig {
            name: RemoteName("fake".into()),
            helper: "fake".into(),
            url: "fake::".into(),
            credentials,
            deadline: None,
            sync: Default::default(),
            path: None,
            except: Vec::new(),
        },
        capabilities: Ok(capabilities),
    }
}

fn report(answers: Option<&[Answer]>) -> Value {
    serde_json::to_value(checked(answers)).unwrap()
}

fn one_row(answer: Answer) -> (Value, Value) {
    let report = report(Some(&[answer]));
    let check = &report[0];
    let row = &check["rows"][0];
    (
        json!([check["result"], check["detail"]]),
        json!([row["name"], row["result"], row["detail"], row["advice"]]),
    )
}

#[test]
fn a_written_credential_is_present_and_untested() {
    assert_eq!(
        one_row(answer(vec![literal("tok")], &["api_token"])),
        (
            json!(["ok", "1 remote: 1 ok"]),
            json!(["fake", "ok", "has its credential, not tested", TRY])
        )
    );
}

#[test]
fn a_missing_credential_is_an_issue_naming_where_it_goes() {
    assert_eq!(
        one_row(answer(vec![], &["api_token"])),
        (
            json!(["issue", "1 remote: 1 issue"]),
            json!(["fake", "issue", "has no api_token", SET_IT])
        )
    );
}

#[test]
fn an_empty_credential_counts_as_missing() {
    assert_eq!(
        one_row(answer(vec![literal("")], &["api_token"])).1,
        json!(["fake", "issue", "api_token is empty", SET_IT])
    );
}

#[test]
fn a_credential_function_is_reported_as_one_dam_did_not_run() {
    let computed = CredentialSpec::Computed {
        remote: RemoteName("fake".into()),
        name: "api_token".into(),
    };
    assert_eq!(
        one_row(answer(vec![computed], &["api_token"])).1,
        json!([
            "fake",
            "ok",
            "api_token comes from a function dam did not run",
            TRY
        ])
    );
}

#[test]
fn a_helper_asking_for_no_credential_needs_none() {
    assert_eq!(
        one_row(answer(vec![], &[])).1,
        json!(["fake", "ok", "needs no credential", null])
    );
}

#[test]
fn credentials_wait_on_a_helper_that_answers() {
    let mut silent = answer(vec![literal("tok")], &["api_token"]);
    silent.capabilities = Err(HelperError::NotFound {
        helper: "fake".into(),
    });
    assert_eq!(
        one_row(silent),
        (
            json!(["not_run", "1 remote: 1 not checked"]),
            json!([
                "fake",
                "not_run",
                "not checked, its helper did not answer",
                null
            ])
        )
    );
}
