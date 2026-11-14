use std::time::Duration;

use dam_protocol::{MutationResult, Verdict};

use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::{FakeTodoist, answered};
use crate::api::{ApiError, SyncResponse, SyncWrite, TodoistSync};
use crate::push::push;

const SERVICE: &str = "Todoist";

fn unanswered_why(result: &MutationResult) -> &str {
    match &result.verdict {
        Verdict::Unanswered { why } => why,
        other => panic!("{} was answered {other:?}", result.oid),
    }
}

fn an_update() -> dam_protocol::Mutation {
    let object = task(&"1".repeat(40), "Work/", "oat", false);
    with_fields(
        mutation("update", 1, Some("i:i1"), Some(object)),
        &["subject"],
    )
}

fn a_create(done: bool) -> dam_protocol::Mutation {
    let object = task(&"1".repeat(40), "Work/", "eggs", done);
    mutation("create", 1, None, Some(object))
}

#[test]
fn each_write_with_no_answer_says_what_happened() {
    let causes = [
        (
            ApiError::TimedOut {
                service: SERVICE,
                after: Duration::from_secs(30),
            },
            "Todoist did not answer within 30s",
        ),
        (
            ApiError::transport(SERVICE, "connection closed before the answer"),
            "the connection to Todoist was lost before it answered: \
             connection closed before the answer",
        ),
        (
            answered(502, "bad gateway"),
            "Todoist answered with a server error, 502: bad gateway",
        ),
        (
            ApiError::decode(SERVICE, "expected value at line 1 column 1"),
            "Todoist's answer could not be read: expected value at line 1 column 1",
        ),
        (
            ApiError::TooLarge {
                service: SERVICE,
                limit: 1,
            },
            "Todoist's answer was past 1 bytes and could not be read",
        ),
    ];
    for (cause, said) in causes {
        let api = FakeTodoist::new(sync_body()).failing_write(1, cause);
        let response = push(&api, vec![an_update()]).unwrap();
        assert_eq!(unanswered_why(&response.results[0]), said);
    }
}

struct TodoistThatGivesNoVerdicts;

impl TodoistSync for TodoistThatGivesNoVerdicts {
    fn read_all(&self) -> Result<SyncResponse, ApiError> {
        serde_json::from_value(sync_body()).map_err(|e| ApiError::decode(SERVICE, e))
    }

    fn read_changes(&self, _since: &str) -> Result<SyncResponse, ApiError> {
        self.read_all()
    }

    fn write(&self, _commands: &[serde_json::Value]) -> Result<SyncWrite, ApiError> {
        Ok(SyncWrite::default())
    }
}

#[test]
fn a_command_todoist_gave_no_verdict_for_is_named() {
    let response = push(&TodoistThatGivesNoVerdicts, vec![an_update()]).unwrap();
    assert_eq!(
        unanswered_why(&response.results[0]),
        "Todoist gave no verdict for item_update"
    );
}

#[test]
fn a_create_todoist_named_no_id_for_says_so() {
    let api = FakeTodoist::new(sync_body());
    push(&api, vec![a_create(false)]).unwrap();
    let resent = push(&api, vec![a_create(false)]).unwrap();
    assert_eq!(
        unanswered_why(&resent.results[0]),
        "Todoist accepted the new task but named no id for it"
    );
}

#[test]
fn a_mutation_todoist_did_only_part_of_names_the_refused_command_and_its_words() {
    let api = FakeTodoist::new(sync_body()).refusing("item_close");
    let response = push(&api, vec![a_create(true)]).unwrap();
    assert_eq!(
        unanswered_why(&response.results[0]),
        "Todoist accepted item_add but refused item_close: refused by the test"
    );
}
