use dam_domain::Permission::{ConflictRead, EventRead, RemotePush, TaskRead};
use dam_domain::{Object, Permission, Task};

use crate::access::Holder;
use crate::ports::{Conflict, Declared};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_caps, task_in, todoist};
use crate::use_cases::push::PushPreview;

fn on_the_wire(_: &crate::remote::RemoteMutation) -> Result<Declared, crate::ports::HelperError> {
    Ok(Declared::new())
}

fn work_and_home() -> Scenario {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "laundry", "home")));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    s.store
        .set_accepted_fields(&todoist(), &task_caps().fields)
        .unwrap();
    s
}

fn in_conflict(s: &Scenario) {
    s.store
        .mark_conflict(&Conflict {
            remote: todoist(),
            oid: oid(2),
            ours: Some(Object::Task(task_in(2, "laundry", "home"))),
            theirs: Object::Task(Task::new(oid(2), "theirs")),
        })
        .unwrap();
}

fn token(permissions: &[Permission], paths: Option<&[&str]>) -> Holder {
    holder(vec![
        allowed(&[RemotePush], None),
        allowed(permissions, paths),
    ])
}

fn previewed_by(s: &Scenario, token: &Holder) -> PushPreview {
    let dam = Dam {
        access: token,
        ..s.dam()
    };
    dam.push_preview(None, &on_the_wire).unwrap().remove(0)
}

fn subjects(preview: &PushPreview) -> Vec<String> {
    let plan = preview.plan.as_ref().unwrap();
    plan.mutations
        .iter()
        .map(|m| m.object.as_ref().unwrap().base().subject.clone())
        .collect()
}

#[test]
fn a_token_that_reads_nothing_learns_nothing_of_the_plan() {
    let s = work_and_home();
    in_conflict(&s);
    let preview = previewed_by(&s, &token(&[], None));
    let plan = preview.plan.as_ref().unwrap();
    assert!(plan.mutations.is_empty());
    assert!(plan.held.is_empty());
    assert_eq!(plan.conflicted, None);
    assert!(plan.skipped.is_empty());
}

#[test]
fn a_token_that_reads_part_sees_its_rows_in_full_and_no_count_of_the_others() {
    let s = work_and_home();
    let preview = previewed_by(&s, &token(&[TaskRead], Some(&["work"])));
    assert_eq!(subjects(&preview), ["report"]);
    assert!(preview.plan.as_ref().unwrap().skipped.is_empty());
}

#[test]
fn a_token_without_conflict_read_gets_no_conflict_list() {
    let s = work_and_home();
    in_conflict(&s);
    let preview = previewed_by(&s, &token(&[TaskRead, EventRead], None));
    assert_eq!(preview.plan.as_ref().unwrap().conflicted, None);
    let preview = previewed_by(&s, &token(&[TaskRead, EventRead, ConflictRead], None));
    assert_eq!(
        preview.plan.as_ref().unwrap().conflicted,
        Some(vec![oid(2)])
    );
}

#[test]
fn the_owner_sees_everything() {
    let s = work_and_home();
    in_conflict(&s);
    let preview = s.dam().push_preview(None, &on_the_wire).unwrap().remove(0);
    assert_eq!(subjects(&preview), ["report"]);
    let plan = preview.plan.as_ref().unwrap();
    assert_eq!(plan.conflicted, Some(vec![oid(2)]));
    assert!(plan.skipped.is_empty());
}
