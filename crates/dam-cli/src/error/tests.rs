use super::*;
use dam_application::EditorError;
use dam_domain::Blocker;

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(byte))
}

#[test]
fn a_refusal_names_the_rule_it_broke_and_another_failure_names_none() {
    let refused = CliError::UseCase(UseCaseError::Refused(Refusal::NothingToCommit));
    assert_eq!(refused.document()["error"]["rule"], "nothing_to_commit");
    let failed = CliError::Io("disk".into());
    assert_eq!(
        failed.document()["error"]["rule"],
        serde_json::Value::Null,
        "only a refusal breaks a rule"
    );
}

#[test]
fn a_blocked_completion_names_the_task_and_then_its_blockers() {
    let error = CliError::UseCase(UseCaseError::Refused(Refusal::Blocked {
        oid: oid(1),
        blockers: vec![Blocker::OpenChild(oid(2)), Blocker::OpenDependency(oid(3))],
    }));
    let document = error.document();
    assert_eq!(document["error"]["kind"], "refused");
    assert_eq!(
        document["error"]["oids"],
        serde_json::json!([oid(1).to_string(), oid(2).to_string(), oid(3).to_string()])
    );
}

#[test]
fn the_message_is_the_sentence_the_human_form_prints() {
    let error = CliError::UseCase(UseCaseError::Refused(Refusal::NotCompleted(oid(4))));
    assert_eq!(error.document()["error"]["message"], error.to_string());
}

#[test]
fn an_ambiguous_prefix_is_a_usage_failure_listing_what_it_matched() {
    let error = CliError::Ambiguous {
        text: "abab".into(),
        matches: vec![oid(5), oid(6)],
    };
    assert_eq!(error.document()["error"]["kind"], "usage");
    assert_eq!(
        error.document()["error"]["oids"],
        serde_json::json!([oid(5).to_string(), oid(6).to_string()])
    );
}

#[test]
fn a_failure_naming_no_object_carries_an_empty_list() {
    let error = CliError::UseCase(UseCaseError::Editor(EditorError("no editor".into())));
    assert_eq!(error.document()["error"]["oids"], serde_json::json!([]));
    assert_eq!(error.document()["error"]["rule"], serde_json::Value::Null);
}

#[test]
fn a_config_file_that_cannot_be_read_is_a_store_failure_and_a_bad_one_a_parse_failure() {
    assert_eq!(
        CliError::Config(ConfigError::Io("x".into())).kind(),
        "store"
    );
    assert_eq!(
        CliError::Config(ConfigError::Syntax("x".into())).kind(),
        "parse"
    );
}
