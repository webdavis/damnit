use dam_domain::Permission::{ConflictRead, RemotePush};
use dam_domain::{Object, Task};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::Conflict;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, pushing, remotes_launched_by, task, todoist};

fn one_conflict() -> Scenario {
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
    s
}

fn refusal_to(s: &Scenario, permissions: &[dam_domain::Permission]) -> UseCaseError {
    let token = holder(vec![allowed(permissions, None)]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let (l, _) = pushing(|_| vec![]);
    dam.push(remotes_launched_by(&l), None).unwrap_err()
}

#[test]
fn a_token_without_conflict_read_is_refused_without_the_count() {
    assert_eq!(
        refusal_to(&one_conflict(), &[RemotePush]),
        UseCaseError::Refused(Refusal::UnresolvedConflicts(None))
    );
}

#[test]
fn a_token_with_conflict_read_is_told_the_count() {
    assert_eq!(
        refusal_to(&one_conflict(), &[RemotePush, ConflictRead]),
        UseCaseError::Refused(Refusal::UnresolvedConflicts(Some(1)))
    );
}
