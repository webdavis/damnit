use dam_domain::{ChildDisposition, DependencyDisposition, Dispositions, Force, Recurrence};
use dam_domain::{Object, Tracked, When};
use jiff::civil::date;

use super::{every_hook, hooked, new_task, ran, retitled};
use crate::config::HookEvent::{TaskChanged, TaskDone, TaskNew};
use crate::ports::History;
use crate::testing::{oid, task_in};

#[test]
fn creating_a_task_runs_task_new_once_with_the_task_as_created() {
    let s = every_hook();
    s.dam().new_task(new_task("call the dentist")).unwrap();
    assert_eq!(ran(&s), [(TaskNew, "call the dentist")]);
}

#[test]
fn editing_a_task_runs_task_changed_with_the_task_as_edited() {
    let s = every_hook();
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    assert_eq!(ran(&s), [(TaskChanged, "pay the rent")]);
}

#[test]
fn completing_a_task_runs_task_done() {
    let s = every_hook();
    s.dam().complete(&oid(1), Force::No, Tracked::No).unwrap();
    assert_eq!(ran(&s), [(TaskDone, "pay rent")]);
}

#[test]
fn completing_a_repeating_task_that_rolls_forward_runs_task_done() {
    let s = every_hook();
    let mut plants = task_in(2, "water the plants", "");
    plants.base.recurrence = Some(Recurrence::parse("every week"));
    plants.due = Some(When::Day(date(2026, 9, 18)));
    s.filed(plants);
    s.dam().complete(&oid(2), Force::No, Tracked::No).unwrap();
    assert_eq!(ran(&s), [(TaskDone, "water the plants")]);
}

#[test]
fn done_that_moves_children_runs_one_hook_for_each_task_it_changed() {
    let s = every_hook();
    s.filed(task_in(2, "plan", "work"))
        .filed(task_in(3, "step", "work/plan"));
    let into_a_group = Force::With(Dispositions {
        children: ChildDisposition::Into("leftovers".into()),
        dependencies: DependencyDisposition::Keep,
    });
    s.dam()
        .complete(&oid(2), into_a_group, Tracked::No)
        .unwrap();
    assert_eq!(
        ran(&s),
        [
            (TaskDone, "plan"),
            (TaskChanged, "step"),
            (TaskNew, "leftovers")
        ]
    );
}

#[test]
fn only_an_event_with_a_hook_runs_one() {
    let s = hooked(&[(TaskDone, None)]);
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    s.dam().new_task(new_task("call the dentist")).unwrap();
    assert_eq!(ran(&s), []);
    s.dam().complete(&oid(1), Force::No, Tracked::No).unwrap();
    assert_eq!(ran(&s), [(TaskDone, "pay the rent")]);
}

#[test]
fn a_hook_reads_the_history_of_the_task_it_was_handed() {
    let s = every_hook();
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    let histories = s.fixture.hooks.histories.borrow();
    let [History::Recorded(commits)] = &histories[..] else {
        panic!("no history read: {histories:?}");
    };
    assert_eq!(commits.len(), 1);
    assert!(matches!(
        commits[0].changes[0].change.after(),
        Some(Object::Task(t)) if t.base.subject == "pay rent"
    ));
}
