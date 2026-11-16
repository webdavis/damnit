use dam_domain::Object;
use dam_domain::Permission::{RemotePush, TaskRead};

use crate::errors::UseCaseError;
use crate::ports::HelperError;
use crate::remote::{MutationOutcome, MutationVerdict, PushOutcome};
use crate::service::Dam;
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, allowed, holder, remotes_launched_by, task_in,
};

fn answering_two_then_cancelled() -> ScriptedLauncher {
    ScriptedLauncher::launching(|| ScriptedHelper {
        push_answer: Box::new(|ms| {
            Ok(PushOutcome {
                outcomes: ms[..2]
                    .iter()
                    .map(|m| MutationOutcome {
                        oid: m.oid.clone(),
                        verdict: MutationVerdict::Accepted { remote_id: None },
                    })
                    .collect(),
                stopped_by: Some(HelperError::Cancelled),
            })
        }),
        ..ScriptedHelper::answering(Ok(Default::default()))
    })
}

fn stopped_for(path: &str) -> (usize, usize) {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "slides", "work")));
    s.committed(Object::Task(task_in(3, "laundry", "home")));
    let token = holder(vec![
        allowed(&[RemotePush], None),
        allowed(&[TaskRead], Some(&[path])),
    ]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let launcher = answering_two_then_cancelled();
    match dam.push(remotes_launched_by(&launcher), None).unwrap_err() {
        UseCaseError::PushStopped {
            kept, unanswered, ..
        } => (kept, unanswered),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_stopped_push_counts_for_a_token_only_what_it_reads() {
    assert_eq!(stopped_for("work"), (2, 0));
    assert_eq!(stopped_for("home"), (0, 1));
}
