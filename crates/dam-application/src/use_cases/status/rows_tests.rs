use dam_domain::Object;
use dam_domain::Permission::{StageRead, TaskRead, WorkingRead};

use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, task_in};

fn staged_and_edited_again() -> Scenario {
    let s = Scenario::default();
    s.committed(Object::Task(task_in(1, "report", "work")));
    s.committed(Object::Task(task_in(2, "laundry", "home")));
    s.working(Object::Task(task_in(1, "report v2", "work")));
    s.working(Object::Task(task_in(2, "laundry v2", "home")));
    s.dam().add_all().unwrap();
    s.working(Object::Task(task_in(1, "report v3", "work")));
    s.working(Object::Task(task_in(2, "laundry v3", "home")));
    s
}

fn rows_for(s: &Scenario, task_read: Option<Option<&[&str]>>) -> (usize, usize) {
    let mut grants = vec![allowed(&[WorkingRead, StageRead], None)];
    if let Some(paths) = task_read {
        grants.push(allowed(&[TaskRead], paths));
    }
    let token = holder(grants);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    (
        dam.seen_staged().unwrap().len(),
        dam.diff_working().unwrap().len(),
    )
}

#[test]
fn a_token_without_task_read_gets_no_working_or_staged_row() {
    assert_eq!(rows_for(&staged_and_edited_again(), None), (0, 0));
}

#[test]
fn a_token_that_reads_one_place_gets_only_that_places_rows() {
    assert_eq!(
        rows_for(&staged_and_edited_again(), Some(Some(&["work"]))),
        (1, 1)
    );
}

#[test]
fn a_token_that_reads_all_and_the_owner_get_every_row() {
    let s = staged_and_edited_again();
    assert_eq!(rows_for(&s, Some(None)), (2, 2));
    let owner = s.dam();
    assert_eq!(
        (
            owner.seen_staged().unwrap().len(),
            owner.diff_working().unwrap().len()
        ),
        (2, 2)
    );
}
