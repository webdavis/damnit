use dam_domain::{Object, Path};

use crate::config::Config;
use crate::ports::{Conflict, Notice, RemoteName};
use crate::remote::{MutationOutcome, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, config, oid, pushing, refusing_all, remotes_launched_by, task,
    task_in, todoist,
};

fn push(
    s: &Scenario,
    answer: impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static,
) {
    let (l, _) = pushing(answer);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
}

fn push_failed(s: &Scenario) -> Vec<(RemoteName, String)> {
    s.store
        .notices()
        .unwrap()
        .into_iter()
        .filter_map(|n| match n {
            Notice::PushFailed {
                remote,
                oid: o,
                why,
            } if o == oid(1) => Some((remote, why)),
            _ => None,
        })
        .collect()
}

fn failed_on_todoist(why: &str) -> Vec<(RemoteName, String)> {
    vec![(todoist(), why.to_string())]
}

fn scoped_to_home() -> Config {
    let mut config = config();
    config.remotes[0].path = Some(Path::parse("Home").unwrap());
    config
}

#[test]
fn an_accepted_push_clears_the_notice_of_the_earlier_failure() {
    let s = Scenario::default();
    s.committed(task(1));
    push(&s, refusing_all("temporary failure"));
    assert_eq!(push_failed(&s), failed_on_todoist("temporary failure"));

    push(&s, accepting_all(Some("r1")));
    assert_eq!(push_failed(&s), vec![]);
}

#[test]
fn a_push_that_fails_again_replaces_the_notice() {
    let s = Scenario::default();
    s.committed(task(1));
    push(&s, refusing_all("first"));
    push(&s, refusing_all("second"));
    assert_eq!(push_failed(&s), failed_on_todoist("second"));
}

#[test]
fn a_push_to_one_remote_leaves_the_notices_of_another() {
    let s = Scenario::default();
    s.committed(task(1));
    let calendar = (RemoteName("calendar".into()), "busy".to_string());
    s.store
        .add_notice(&Notice::PushFailed {
            remote: calendar.0.clone(),
            oid: oid(1),
            why: calendar.1.clone(),
        })
        .unwrap();
    push(&s, accepting_all(Some("r1")));
    assert_eq!(push_failed(&s), vec![calendar]);
}

#[test]
fn an_object_held_for_a_conflict_keeps_its_notice() {
    let s = Scenario::default();
    s.committed(task(1));
    push(&s, refusing_all("temporary failure"));
    let ours = s.store.committed(&oid(1)).unwrap().unwrap();
    let mut theirs = ours.clone();
    theirs.base_mut().subject = "theirs".into();
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(1),
            ours: Some(ours),
            theirs,
        })
        .unwrap();

    push(&s, accepting_all(Some("r1")));
    assert_eq!(push_failed(&s), failed_on_todoist("temporary failure"));
}

#[test]
fn an_object_removed_here_before_its_create_landed_loses_its_notice() {
    let s = Scenario::default();
    s.committed(task(1));
    push(&s, refusing_all("temporary failure"));
    s.deleted(&oid(1));

    push(&s, accepting_all(Some("r1")));
    assert_eq!(push_failed(&s), vec![]);
}

#[test]
fn an_object_that_no_longer_goes_to_the_remote_loses_its_notice() {
    let s = Scenario::default().configured(scoped_to_home());
    s.committed(Object::Task(task_in(1, "t1", "Home/")));
    push(&s, refusing_all("temporary failure"));
    s.committed(Object::Task(task_in(1, "t1", "Work/")));

    push(&s, accepting_all(Some("r1")));
    assert_eq!(push_failed(&s), vec![]);
}
