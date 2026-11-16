use super::tests::args;
use super::*;
use crate::args::FieldArgs;
use crate::testing::context;
use dam_domain::When;
use jiff::civil::date;

fn made(ctx: &Context) -> Object {
    ctx.store.all().unwrap().remove(0)
}

#[test]
fn a_task_takes_today_with_a_count_for_due_deadline_and_focus() {
    let ctx = context();
    let task = NewArgs {
        due: Some("today+2".into()),
        deadline: Some("today+5".into()),
        fields: FieldArgs {
            focus: Some("today+1".into()),
            ..FieldArgs::default()
        },
        ..args("milk")
    };
    run(&ctx, task).unwrap();
    let Object::Task(task) = made(&ctx) else {
        panic!("not a task")
    };
    assert_eq!(task.due, Some(When::Day(date(2026, 9, 20))));
    assert_eq!(task.deadline, Some(date(2026, 9, 23)));
    assert_eq!(task.focus.map(|f| f.day), Some(date(2026, 9, 19)));
}

#[test]
fn an_event_takes_today_with_a_count_and_now_with_a_length() {
    for (start, end) in [("today", "today+1"), ("now+1h", "now+2h")] {
        let ctx = context();
        let event = NewArgs {
            kind: Kind::Event,
            start: Some(start.into()),
            end: Some(end.into()),
            ..args("offsite")
        };
        run(&ctx, event).unwrap();
        let Object::Event(event) = made(&ctx) else {
            panic!("not an event")
        };
        assert!(event.start.date() <= event.end.date(), "{start} {end}");
    }
}

#[test]
fn a_period_given_as_a_date_to_set_is_refused_and_writes_nothing() {
    let ctx = context();
    let event = NewArgs {
        kind: Kind::Event,
        start: Some("today".into()),
        end: Some("week".into()),
        ..args("offsite")
    };
    let Err(CliError::Usage(text)) = run(&ctx, event) else {
        panic!("not refused")
    };
    assert!(text.contains("one day or instant"), "{text}");
    assert!(ctx.store.all().unwrap().is_empty());
}
