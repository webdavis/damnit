use dam_domain::Permission::{self, RemotePull, RemotePush};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, allowed, holder, pushing, remotes_launched_by, task, todoist,
};

fn sync_holding(permissions: &[Permission]) -> (Scenario, Result<(), UseCaseError>, usize) {
    let s = Scenario::default();
    s.committed(task(1));
    let token = holder(vec![allowed(permissions, None)]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let (launcher, pushed) = pushing(accepting_all(None));
    let result = dam
        .sync_now(remotes_launched_by(&launcher), "todoist")
        .map(|_| ());
    let sent = pushed.borrow().len();
    (s, result, sent)
}

fn assert_untouched(s: &Scenario, sent: usize) {
    assert_eq!(sent, 0);
    assert!(s.store.last_pull(&todoist()).unwrap().is_none());
    assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 1);
}

#[test]
fn a_token_that_may_only_pull_is_refused_before_anything_is_pulled() {
    let (s, result, sent) = sync_holding(&[RemotePull]);
    assert_eq!(
        result,
        Err(UseCaseError::Refused(Refusal::NotPermitted(RemotePush)))
    );
    assert_untouched(&s, sent);
}

#[test]
fn a_token_that_may_only_push_is_refused_before_anything_is_pulled() {
    let (s, result, sent) = sync_holding(&[RemotePush]);
    assert_eq!(
        result,
        Err(UseCaseError::Refused(Refusal::NotPermitted(RemotePull)))
    );
    assert_untouched(&s, sent);
}

#[test]
fn a_token_that_may_pull_and_push_syncs() {
    let (s, result, sent) = sync_holding(&[RemotePull, RemotePush]);
    assert_eq!(result, Ok(()));
    assert_eq!(sent, 1);
    assert!(s.store.last_pull(&todoist()).unwrap().is_some());
}
