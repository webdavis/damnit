use crate::args::StatusArgs;
use crate::testing::{context, holding};
use dam_application::{Conflict, RemoteName};
use dam_domain::Permission::{ConflictRead, TaskRead};
use dam_domain::{Object, Oid, Task};

const UNSEEN: &str =
    "some changes you cannot see did not go through; ask the owner to run dam status";

fn a_conflict_on_a_task() -> crate::context::Context {
    let ctx = context();
    let task = Object::Task(Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(1)), "t"));
    ctx.store.put(&task).unwrap();
    ctx.store
        .mark_conflict(&Conflict {
            remote: RemoteName("todoist".into()),
            oid: task.oid().clone(),
            ours: Some(task.clone()),
            theirs: task,
        })
        .unwrap();
    ctx
}

#[test]
fn a_token_that_cannot_read_a_conflicted_task_gets_only_the_sentence() {
    let mut ctx = a_conflict_on_a_task();
    ctx.access = holding(&[ConflictRead]);
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    assert!(report.human.contains(UNSEEN), "{}", report.human);
    assert_eq!(report.data["unseen_failed"], serde_json::Value::Bool(true));
    assert_eq!(report.data["conflicts"].as_array().unwrap().len(), 0);
}

#[test]
fn a_token_that_reads_the_task_gets_no_sentence() {
    let mut ctx = a_conflict_on_a_task();
    ctx.access = holding(&[ConflictRead, TaskRead]);
    let report = super::run_status(&ctx, StatusArgs { full: false }).unwrap();
    assert!(!report.human.contains(UNSEEN), "{}", report.human);
    assert!(report.data.get("unseen_failed").is_none());
    assert_eq!(report.data["conflicts"].as_array().unwrap().len(), 1);
}
