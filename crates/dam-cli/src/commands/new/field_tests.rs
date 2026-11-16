use super::tests::args;
use super::*;
use crate::args::FieldArgs;
use crate::testing::context;

fn made(ctx: &Context) -> Object {
    ctx.store.all().unwrap().remove(0)
}

#[test]
fn a_task_takes_the_fields_edit_sets() {
    let ctx = context();
    run(
        &ctx,
        NewArgs {
            fields: FieldArgs {
                recurrence: Some("every week".into()),
                estimate: Some(30),
                focus: Some("today".into()),
                ..FieldArgs::default()
            },
            ..args("milk")
        },
    )
    .unwrap();
    let Object::Task(task) = made(&ctx) else {
        panic!("not a task")
    };
    assert!(task.base.recurrence.is_some());
    assert_eq!(task.estimate, Some(30));
    assert!(task.focus.is_some());
}

#[test]
fn an_event_takes_a_location() {
    let ctx = context();
    run(
        &ctx,
        NewArgs {
            kind: Kind::Event,
            start: Some("2026-09-25T14:00".into()),
            end: Some("2026-09-25T15:00".into()),
            fields: FieldArgs {
                location: Some("room 4".into()),
                ..FieldArgs::default()
            },
            ..args("dentist")
        },
    )
    .unwrap();
    let Object::Event(event) = made(&ctx) else {
        panic!("not an event")
    };
    assert_eq!(event.location.as_deref(), Some("room 4"));
}

#[test]
fn a_task_given_a_location_is_a_usage_error_naming_kind_event() {
    let ctx = context();
    let placed = NewArgs {
        fields: FieldArgs {
            location: Some("room 4".into()),
            ..FieldArgs::default()
        },
        ..args("milk")
    };
    let Err(CliError::Usage(text)) = run(&ctx, placed) else {
        panic!("not refused")
    };
    assert!(
        text.contains("--location") && text.contains("--kind event"),
        "{text}"
    );
    assert!(ctx.store.all().unwrap().is_empty());
}

#[test]
fn an_event_given_a_task_field_is_a_usage_error_naming_the_flag() {
    let ctx = context();
    let repeated = NewArgs {
        kind: Kind::Event,
        start: Some("2026-09-25T14:00".into()),
        end: Some("2026-09-25T15:00".into()),
        fields: FieldArgs {
            estimate: Some(30),
            ..FieldArgs::default()
        },
        ..args("dentist")
    };
    let Err(CliError::Usage(text)) = run(&ctx, repeated) else {
        panic!("not refused")
    };
    assert!(text.contains("--estimate"), "{text}");
}

#[test]
fn a_bad_recurrence_is_refused_and_writes_nothing() {
    let ctx = context();
    let repeated = NewArgs {
        fields: FieldArgs {
            recurrence: Some("whenever".into()),
            ..FieldArgs::default()
        },
        ..args("milk")
    };
    assert!(run(&ctx, repeated).is_err());
    assert!(ctx.store.all().unwrap().is_empty());
}
