use super::super::*;
use super::task_of;
use crate::testing::{Scenario, oid, task_in};
use dam_domain::{Completion, Recurrence, When};
use jiff::civil::date;

#[test]
fn a_recurring_task_rolls_forward_instead_of_closing() {
    let s = Scenario::default().drawing_from(9);
    let mut plants = task_in(1, "water the plants", "");
    plants.base.recurrence = Some(Recurrence::parse("every week"));
    plants.due = Some(When::Day(date(2026, 9, 18)));
    s.filed(plants);
    let id = oid(1);
    let out = s.dam().complete(&id, Force::No, Tracked::No).unwrap();
    assert_eq!(
        out,
        Completed::RolledForward {
            next_due: date(2026, 9, 25)
        }
    );
    let t = task_of(&s, &id);
    assert!(!t.completion.is_done());
    assert_eq!(t.due, Some(When::Day(date(2026, 9, 25))));
}

#[test]
fn a_rule_dam_cannot_read_is_refused_and_the_task_stays_open() {
    let s = Scenario::default().drawing_from(9);
    let mut plants = task_in(1, "water the plants", "");
    plants.base.recurrence = Some(Recurrence::parse("every other week"));
    s.filed(plants);
    let id = oid(1);
    let dam = s.dam();
    let refused = |tracked| dam.complete(&id, Force::No, tracked).unwrap_err();
    assert_eq!(
        refused(Tracked::No),
        UseCaseError::Refused(Refusal::UnreadableRecurrence {
            rule: "every other week".into(),
            tracked: Tracked::No,
        })
    );
    assert_eq!(
        refused(Tracked::ByRemote),
        UseCaseError::Refused(Refusal::UnreadableRecurrence {
            rule: "every other week".into(),
            tracked: Tracked::ByRemote,
        })
    );
    assert!(!task_of(&s, &id).completion.is_done());
}

#[test]
fn a_task_that_rolls_forward_records_no_completion_time() {
    let s = Scenario::default().drawing_from(1);
    let mut plants = task_in(1, "water the plants", "");
    plants.base.recurrence = Some(Recurrence::parse("every day"));
    plants.due = Some(When::Day(date(2026, 9, 18)));
    s.filed(plants);
    s.dam().complete(&oid(1), Force::No, Tracked::No).unwrap();
    assert_eq!(task_of(&s, &oid(1)).completion, Completion::Open);
}
