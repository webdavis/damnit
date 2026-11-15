use std::collections::BTreeMap;

use dam_domain::{Channel, Details, Permission::*, Timestamp};

use super::edits::requesting;
use super::*;
use crate::access::{Access, Detailed, Owner};
use crate::ports::CommitRepository;
use crate::service::Dam;
use crate::testing::{allowed, holder};

fn hook_commit_host_when_pulled_by(caller: Box<dyn Access>) -> Option<String> {
    let s = pulled_hooked();
    tracked(&s, 1, "old");
    s.pushed();
    requesting(&s, "filed");
    let access = Detailed {
        caller,
        details: Details {
            meta: BTreeMap::new(),
            host: Some("the caller's host".into()),
            dir: None,
            repo: None,
            branch: None,
            at: Timestamp::UNIX_EPOCH,
            channel: Channel::Local,
        },
    };
    let dam = Dam {
        access: &access,
        ..s.dam()
    };
    let l = pulling(PullOutcome {
        objects: vec![incoming("new", "r1")],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    });
    dam.pull(remotes_launched_by(&l), None).unwrap();
    let hook = &s.store.log().unwrap()[0];
    assert_eq!(hook.message, "task_pulled after pull from todoist");
    hook.changes[0].details.as_ref().unwrap().host.clone()
}

#[test]
fn the_hook_commit_carries_the_owners_details_but_not_a_tokens() {
    assert_eq!(
        hook_commit_host_when_pulled_by(Box::new(Owner)),
        Some("the caller's host".into())
    );
    let token = holder(vec![allowed(
        &[RemotePull, TaskRead, TaskChange, TaskMove],
        None,
    )]);
    assert_eq!(hook_commit_host_when_pulled_by(Box::new(token)), None);
}
