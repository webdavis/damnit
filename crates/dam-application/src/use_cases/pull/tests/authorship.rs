use super::incoming;
use crate::access::{Access, Holder};
use crate::remote::PullOutcome;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, pulling, remotes_launched_by};
use dam_domain::{Author, Authored, Change, CommitId, CommitRecord, Object, Permission, Task};

fn pull_token() -> Holder {
    holder(vec![allowed(&[Permission::RemotePull], None)])
}

fn pulled_in(s: &Scenario, access: &dyn Access, subject: &str, remote_id: &str) {
    let l = pulling(PullOutcome {
        objects: vec![incoming(subject, remote_id)],
        ..PullOutcome::default()
    });
    let dam = Dam { access, ..s.dam() };
    dam.pull(remotes_launched_by(&l), None).unwrap();
}

fn made(token: Holder, s: &Scenario) -> Vec<dam_domain::Oid> {
    let token = token.knowing_what_it_made(&s.store).unwrap();
    token.grants.made.into_iter().collect()
}

#[test]
fn a_task_a_tokens_pull_brings_in_is_the_owners_and_not_the_tokens() {
    let s = Scenario::default().drawing_from(42);
    pulled_in(&s, &pull_token(), "from upstream", "r1");
    let origins = s.dam().origins().unwrap();
    assert_eq!(origins[&oid(42)].author, Author::Owner);
    assert_eq!(origins[&oid(42)].details, None);
    assert!(made(pull_token(), &s).is_empty());
}

#[test]
fn a_task_a_token_made_stays_its_own_once_pulled_back() {
    let s = Scenario::default();
    let token = pull_token();
    let created = Authored {
        author: token.author(),
        ..Authored::by_owner(Change::Create {
            after: Object::Task(Task::new(oid(1), "chore")),
        })
    };
    s.store
        .commit(&CommitRecord {
            id: CommitId::generate(&mut |b: &mut [u8]| b.fill(3)),
            message: "work".into(),
            pulled: false,
            at: jiff::Timestamp::UNIX_EPOCH,
            changes: vec![created],
        })
        .unwrap();
    s.store
        .put(&Object::Task(Task::new(oid(1), "chore")))
        .unwrap();
    s.tracked_as(&oid(1), "r1");
    pulled_in(&s, &crate::access::Owner, "chore upstream", "r1");
    assert_eq!(s.store.log().unwrap()[0].changes.len(), 1);
    assert_eq!(s.dam().origins().unwrap()[&oid(1)].author, token.author());
    assert_eq!(made(token, &s), [oid(1)]);
}
