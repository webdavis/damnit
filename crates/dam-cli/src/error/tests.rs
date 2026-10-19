use super::*;
use crate::editor::EditorError;
use dam_domain::{Blockers, Oid, QueryError};

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
        blockers: Blockers {
            dependencies: vec![oid(2)],
            children: vec![oid(3)],
        },
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
    let error = CliError::from(Refusal::AmbiguousPrefix {
        text: "abab".into(),
        matches: vec![oid(5).to_string(), oid(6).to_string()],
    });
    assert_eq!(error.exit_code(), 2);
    assert_eq!(error.document()["error"]["kind"], "usage");
    assert_eq!(error.document()["error"]["rule"], "ambiguous_prefix");
    assert_eq!(
        error.document()["error"]["oids"],
        serde_json::json!([oid(5).to_string(), oid(6).to_string()])
    );
}

#[test]
fn a_prefix_that_is_not_an_oid_is_a_usage_failure() {
    let error = CliError::UseCase(UseCaseError::NotAnOid("f5".into()));
    assert_eq!(error.exit_code(), 2);
    assert_eq!(error.kind(), "usage");
}

#[test]
fn a_failure_naming_no_object_carries_an_empty_list() {
    let error = CliError::Editor(EditorError("no editor".into()));
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

#[test]
fn a_broken_invariant_is_internal_and_a_failed_read_or_write_is_store() {
    let internal = CliError::Internal("the new object did not persist".into());
    assert_eq!(internal.exit_code(), 1);
    assert_eq!(internal.document()["error"]["kind"], "internal");
    assert_eq!(internal.to_string(), "the new object did not persist");
    assert_eq!(CliError::Io("disk".into()).kind(), "store");
}

#[test]
fn every_kind_the_readme_lists_is_one_dam_reports() {
    let readme = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"));
    let kinds = [
        CliError::UseCase(UseCaseError::Refused(Refusal::NothingToCommit)),
        CliError::Io("x".into()),
        CliError::UseCase(UseCaseError::Helper(
            dam_application::HelperError::Cancelled,
        )),
        CliError::UseCase(UseCaseError::Credential(
            dam_application::CredentialError::Missing("x".into()),
        )),
        CliError::Editor(EditorError("x".into())),
        CliError::UseCase(UseCaseError::Parse("x".into())),
        CliError::Usage("x".into()),
        CliError::Cancelled,
        CliError::Internal("x".into()),
    ]
    .map(|e| format!("`{}`", e.kind()));
    let sentence: String = readme
        .lines()
        .skip_while(|l| !l.starts_with("`kind` is one of"))
        .take(2)
        .collect::<Vec<&str>>()
        .join(" ");
    for kind in &kinds {
        assert!(sentence.contains(kind.as_str()), "{sentence} lacks {kind}");
    }
}

#[test]
fn a_query_dam_cannot_read_exits_two_and_is_of_kind_parse() {
    let error = CliError::UseCase(UseCaseError::Query(QueryError::MissingOperator(
        "due:tomorrow".into(),
    )));
    assert_eq!(error.exit_code(), 2);
    assert_eq!(error.document()["error"]["kind"], "parse");
    assert_eq!(error.to_string(), "expected & or | before due:tomorrow");
}
