use super::*;
use crate::config::HookEvent::{TaskChanged, TaskNew, TaskPulled};
use crate::ports::{CommitRepository, HookMarkRepository, StageRepository};
use crate::testing::oid;

#[test]
fn an_edit_lands_as_a_working_change_that_is_neither_staged_nor_committed() {
    let s = every_hook();
    s.mapped_as(&oid(1), "r1");
    requesting(&s, "pay the rent");
    let run = s.dam().run_hook_over_listed(TaskPulled, None).unwrap();
    assert_eq!(subject(&s, &oid(1)), "pay the rent");
    let committed = s.store.committed(&oid(1)).unwrap().unwrap();
    assert_eq!(committed.base().subject, "pay rent");
    assert!(s.store.staged().unwrap().is_empty());
    assert_eq!(s.store.log().unwrap().len(), 3);
    assert_eq!((run.ran, run.changed), (1, 1));
    assert_eq!(
        *s.fixture.hook_edits.edited.borrow(),
        [(TaskPulled, oid(1))]
    );
}

#[test]
fn a_changed_task_is_marked_with_the_hook_so_its_record_names_it() {
    let s = every_hook();
    requesting(&s, "buy oat milk");
    s.dam()
        .run_hook_over_listed(TaskNew, Some("path:errands"))
        .unwrap();
    assert_eq!(s.store.hooked(&oid(3)).unwrap(), Some(TaskNew));
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), None);
}

#[test]
fn the_hooks_edit_runs_no_other_hook() {
    let s = every_hook();
    requesting(&s, "buy oat milk");
    s.dam()
        .run_hook_over_listed(TaskNew, Some("path:errands"))
        .unwrap();
    assert_eq!(subject(&s, &oid(3)), "buy oat milk");
    assert_eq!(ran(&s), [(TaskNew, "buy milk".to_string())]);
    assert!(!ran(&s).iter().any(|(event, _)| *event == TaskChanged));
}
