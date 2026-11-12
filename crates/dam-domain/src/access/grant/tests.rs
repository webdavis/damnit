use super::*;
use crate::{Event, Oid, Task, When};

fn task(path: &str, labels: &[&str]) -> Object {
    let mut task = Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(1)), "t");
    task.base.path = Path::parse(path).unwrap();
    task.base.labels = labels.iter().map(|l| l.to_string()).collect();
    Object::Task(task)
}

fn event(path: &str) -> Object {
    let start = When::Day(jiff::civil::date(2026, 11, 11));
    let mut event = Event::new(
        Oid::generate(&mut |b: &mut [u8]| b.fill(2)),
        "e",
        start.clone(),
        start,
    );
    event.base.path = Path::parse(path).unwrap();
    Object::Event(event)
}

fn grant(allow: &[Permission]) -> Grant {
    Grant {
        allow: allow.iter().copied().collect(),
        ..Grant::default()
    }
}

fn paths(prefixes: &[&str]) -> Option<Vec<Path>> {
    Some(prefixes.iter().map(|p| Path::parse(p).unwrap()).collect())
}

fn holding(grants: Vec<Grant>) -> Grants {
    let token = crate::Token::generate(&mut |b: &mut [u8]| b.fill(3));
    Grants {
        holder: token.key_id().clone(),
        grants,
    }
}

#[test]
fn nothing_is_permitted_that_no_grant_lists() {
    let g = holding(vec![grant(&[Permission::TaskRead])]);
    assert!(!g.permits(Permission::TaskChange, &[Subject::Object(&task("", &[]))]));
    assert!(!holding(vec![]).permits(Permission::TaskRead, &[Subject::Whole]));
}

#[test]
fn a_request_touching_nothing_is_denied() {
    assert!(!holding(vec![grant(&[Permission::TaskRead])]).permits(Permission::TaskRead, &[]));
}

#[test]
fn paths_match_whole_segments() {
    let g = holding(vec![Grant {
        paths: paths(&["work"]),
        ..grant(&[Permission::TaskRead])
    }]);
    let read = |o: &Object| g.permits(Permission::TaskRead, &[Subject::Object(o)]);
    assert!(read(&task("work", &[])));
    assert!(read(&task("work/acme", &[])));
    assert!(!read(&task("workshop", &[])));
    assert!(!read(&task("", &[])));
}

#[test]
fn labels_match_when_the_object_carries_any_of_them() {
    let g = holding(vec![Grant {
        labels: Some(["agent".into(), "vpt".into()].into()),
        ..grant(&[Permission::TaskRead])
    }]);
    let read = |o: &Object| g.permits(Permission::TaskRead, &[Subject::Object(o)]);
    assert!(read(&task("", &["vpt", "home"])));
    assert!(!read(&task("", &["home"])));
}

#[test]
fn every_restriction_in_one_grant_must_hold() {
    let g = holding(vec![Grant {
        paths: paths(&["work"]),
        labels: Some(["agent".into()].into()),
        ..grant(&[Permission::TaskRead])
    }]);
    let read = |o: &Object| g.permits(Permission::TaskRead, &[Subject::Object(o)]);
    assert!(read(&task("work", &["agent"])));
    assert!(!read(&task("work", &[])));
    assert!(!read(&task("home", &["agent"])));
}

#[test]
fn any_one_grant_may_allow() {
    let g = holding(vec![
        Grant {
            paths: paths(&["work"]),
            ..grant(&[Permission::TaskRead])
        },
        Grant {
            paths: paths(&["home"]),
            ..grant(&[Permission::TaskRead])
        },
    ]);
    assert!(g.permits(Permission::TaskRead, &[Subject::Object(&task("home", &[]))]));
    assert!(g.permits(Permission::TaskRead, &[Subject::Object(&task("work", &[]))]));
}

#[test]
fn one_grant_must_cover_every_object_a_request_touches_before_and_after() {
    let g = holding(vec![
        Grant {
            paths: paths(&["work"]),
            ..grant(&[Permission::TaskMove])
        },
        Grant {
            paths: paths(&["home"]),
            ..grant(&[Permission::TaskMove])
        },
    ]);
    let (before, inside, outside) = (task("work", &[]), task("work/a", &[]), task("home", &[]));
    let mv = |after: &Object| {
        g.permits(
            Permission::TaskMove,
            &[Subject::Object(&before), Subject::Object(after)],
        )
    };
    assert!(mv(&inside));
    assert!(!mv(&outside));
}

#[test]
fn own_matches_nothing_until_authorship_is_recorded() {
    let g = holding(vec![Grant {
        own: true,
        ..grant(&[Permission::TaskRead])
    }]);
    assert!(!g.permits(Permission::TaskRead, &[Subject::Object(&task("", &[]))]));
}

#[test]
fn a_task_permission_never_covers_an_event() {
    let g = holding(vec![grant(&[Permission::TaskRead])]);
    assert!(!g.permits(Permission::TaskRead, &[Subject::Object(&event(""))]));
    assert!(g.permits(Permission::TaskRead, &[Subject::Object(&task("", &[]))]));
}

#[test]
fn remotes_restrict_remote_requests_by_name() {
    let g = holding(vec![Grant {
        remotes: Some(vec!["todoist".into()]),
        ..grant(&[Permission::RemotePull])
    }]);
    assert!(g.permits(Permission::RemotePull, &[Subject::Remote("todoist")]));
    assert!(!g.permits(Permission::RemotePull, &[Subject::Remote("gcal")]));
    let any = holding(vec![grant(&[Permission::RemotePull])]);
    assert!(any.permits(Permission::RemotePull, &[Subject::Remote("gcal")]));
}

#[test]
fn a_restriction_that_cannot_describe_the_subject_denies() {
    let by_path = holding(vec![Grant {
        paths: paths(&["work"]),
        ..grant(&[Permission::HistoryRead, Permission::RemotePush])
    }]);
    assert!(!by_path.permits(Permission::HistoryRead, &[Subject::Whole]));
    assert!(!by_path.permits(Permission::RemotePush, &[Subject::Remote("todoist")]));
    let by_remote = holding(vec![Grant {
        remotes: Some(vec!["todoist".into()]),
        ..grant(&[Permission::TaskRead])
    }]);
    assert!(!by_remote.permits(Permission::TaskRead, &[Subject::Object(&task("", &[]))]));
    assert!(
        holding(vec![grant(&[Permission::HistoryRead])])
            .permits(Permission::HistoryRead, &[Subject::Whole])
    );
}

#[test]
fn holds_asks_only_whether_some_grant_lists_the_permission() {
    let g = holding(vec![Grant {
        own: true,
        ..grant(&[Permission::TaskRead])
    }]);
    assert!(g.holds(Permission::TaskRead));
    assert!(!g.holds(Permission::EventRead));
}
