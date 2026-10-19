use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Conflict, HelperError};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, config, oid, pushing, remotes_launched_by, task,
    task_caps, todoist,
};
use dam_domain::{Object, Task};

#[test]
fn unresolved_conflicts_block_push() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(1),
            ours: task(1),
            theirs: Object::Task(Task::new(oid(1), "theirs")),
        })
        .unwrap();
    let (l, _) = pushing(|_| vec![]);
    let err = s.dam().push(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::UnresolvedConflicts(1)));
}

#[test]
fn an_unknown_remote_is_refused() {
    let s = Scenario::default();
    let (l, _) = pushing(|_| vec![]);
    let err = s
        .dam()
        .push(remotes_launched_by(&l), Some("nope"))
        .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
    );
}

#[test]
fn a_credential_the_config_does_not_supply_is_refused_by_name() {
    let mut config = config();
    config.remotes[0].credentials.clear();
    let s = Scenario::default().configured(config);
    let (l, _) = pushing(|_| vec![]);
    let err = s.dam().push(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into()
        })
    );
    assert_eq!(
        l.launched_with.borrow().len(),
        1,
        "one process, even when the capabilities it reported cannot be met"
    );
}

#[test]
fn a_helper_that_times_out_leaves_the_store_as_it_was() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    let before = s.store.clone();
    let timeout = HelperError::Timeout {
        helper: "todoist".into(),
        deadline: "30s".into(),
        said: None,
    };
    let answer = timeout.clone();
    let l = ScriptedLauncher::launching(move || {
        let answer = answer.clone();
        ScriptedHelper {
            push_answer: Box::new(move |_| Err(answer.clone())),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    });
    let err = s.dam().push(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(err, UseCaseError::Helper(timeout));
    assert_eq!(s.store, before);
}
