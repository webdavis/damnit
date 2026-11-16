use dam_domain::{Object, Oid, Task, When};
use jiff::civil::date;

use super::run_agenda;
use super::tests::{standup, timed};
use crate::testing::{agenda_args, context};

#[test]
fn each_row_names_the_day_it_falls_on_and_the_answer_names_the_windows_days() {
    let ctx = context();
    ctx.store.put(&Object::Event(standup())).unwrap();
    let late = timed(
        2,
        "late",
        "2026-09-19T23:30:00+00:00[UTC]",
        "2026-09-20T01:00:00+00:00[UTC]",
    );
    ctx.store.put(&late).unwrap();
    let mut rent = Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(5)), "rent");
    rent.due = Some(When::Day(date(2026, 9, 20)));
    ctx.store.put(&Object::Task(rent)).unwrap();
    let words = ["--kind", "any", "--from", "today", "--to", "week"];
    let answer = run_agenda(&ctx, agenda_args(&words)).unwrap().data;
    assert_eq!(
        (&answer["from"], &answer["to"]),
        (&"2026-09-18".into(), &"2026-09-20".into())
    );
    let days = |key: &str| -> Vec<(String, String)> {
        let rows = answer[key].as_array().unwrap();
        rows.iter()
            .map(|r| {
                (
                    r["day"].as_str().unwrap().into(),
                    r["weekday"].as_str().unwrap().into(),
                )
            })
            .collect()
    };
    assert_eq!(
        days("events"),
        [
            ("2026-09-18".into(), "fri".into()),
            ("2026-09-19".into(), "sat".into())
        ]
    );
    assert_eq!(days("tasks"), [("2026-09-20".into(), "sun".into())]);
}
