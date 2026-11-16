use std::collections::BTreeSet;

use dam_domain::Permission::{TaskCreate, TaskRead};
use dam_domain::{Change, Object, Path, Priority};

use crate::config::{Config, HookConfig, HookEvent};
use crate::service::Dam;
use crate::testing::{Scenario, allowed, config, holder, oid, task_in};
use crate::use_cases::new_object::NewTask;

fn hooked() -> Scenario {
    let s = Scenario::default().configured(Config {
        hooks: vec![HookConfig {
            event: HookEvent::TaskNew,
            on_failure: None,
            file: String::new(),
        }],
        ..config()
    });
    s.working(Object::Task(task_in(1, "report", "work")));
    s
}

fn milk(path: &str) -> NewTask {
    NewTask {
        parent: None,
        subject: "milk".into(),
        path: Path::parse(path).unwrap(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: BTreeSet::new(),
        body: String::new(),
        id: None,
    }
}

#[test]
fn a_rehearsal_answers_the_changes_and_writes_nothing_and_runs_no_hook() {
    let s = hooked();
    let before = s.store.clone();
    let rehearsed = s.dam().rehearse(|dam| dam.new_task(milk("home"))).unwrap();
    assert!(matches!(
        rehearsed.changes.as_slice(),
        [Change::Create { after }] if after.base().subject == "milk"
    ));
    assert_eq!(s.store, before);
    assert!(s.fixture.hooks.ran().is_empty());
}

#[test]
fn a_refused_rehearsal_refuses_as_the_run_would() {
    let s = hooked();
    let before = s.store.clone();
    let mut task = milk("home");
    task.parent = Some(oid(9));
    assert!(s.dam().rehearse(|dam| dam.new_task(task)).is_err());
    assert_eq!(s.store, before);
}

#[test]
fn a_token_learns_no_change_to_an_object_it_cannot_see() {
    let s = hooked();
    let token = holder(vec![allowed(&[TaskRead, TaskCreate], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let rehearsed = dam
        .rehearse(|_| {
            s.dam().new_task(milk("work"))?;
            Ok(())
        })
        .unwrap();
    assert!(rehearsed.changes.is_empty());
}

#[test]
fn a_token_sees_the_changes_to_what_it_sees() {
    let s = hooked();
    let token = holder(vec![allowed(
        &[TaskRead, TaskCreate],
        Some(&["home", "work"]),
    )]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let rehearsed = dam
        .rehearse(|_| {
            s.dam().new_task(milk("work"))?;
            Ok(())
        })
        .unwrap();
    assert_eq!(rehearsed.changes.len(), 1);
}
