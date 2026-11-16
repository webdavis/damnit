use dam_domain::Object;

use super::run_agenda;
use super::tests::{standup, subjects, timed};
use crate::context::Context;
use crate::output::Report;
use crate::testing::{agenda_args, context};

fn day_of_events() -> Context {
    let ctx = context();
    ctx.store.put(&Object::Event(standup())).unwrap();
    for event in [
        timed(
            2,
            "late",
            "2026-09-18T23:30:00+00:00[UTC]",
            "2026-09-19T00:00:00+00:00[UTC]",
        ),
        timed(
            3,
            "next",
            "2026-09-19T00:00:00+00:00[UTC]",
            "2026-09-19T00:30:00+00:00[UTC]",
        ),
    ] {
        ctx.store.put(&event).unwrap();
    }
    ctx
}

fn agenda(ctx: &Context, words: &[&str]) -> Report {
    run_agenda(ctx, agenda_args(words)).unwrap()
}

#[test]
fn a_selection_flag_narrows_the_events() {
    let ctx = day_of_events();
    let narrowed = agenda(&ctx, &["--labels", "work", "--to", "week"]);
    assert_eq!(subjects(&narrowed), ["standup"]);
}

#[test]
fn a_day_as_to_counts_whole_and_an_instant_does_not() {
    let ctx = day_of_events();
    let today = agenda(&ctx, &["--to", "today"]);
    assert_eq!(subjects(&today), ["standup", "late"]);
    let tomorrow = agenda(&ctx, &["--from", "today+1", "--to", "today+1"]);
    assert_eq!(subjects(&tomorrow), ["next"]);
    let morning = agenda(&ctx, &["--to", "2026-09-18T14:00"]);
    assert!(subjects(&morning).is_empty());
}

#[test]
fn a_task_field_exits_2_and_points_at_list() {
    let ctx = day_of_events();
    let err = run_agenda(&ctx, agenda_args(&["--due", "today"])).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert_eq!(
        err.to_string(),
        "--due holds only for tasks, and agenda lists events; run dam list --due today"
    );
}

#[test]
fn a_bad_selection_value_names_agenda_in_the_command_to_run() {
    let ctx = day_of_events();
    let err = run_agenda(&ctx, agenda_args(&["--start", "tomorow"])).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string().ends_with("; run dam agenda --start today"),
        "{err}"
    );
}

#[test]
fn a_bad_window_edge_names_the_shapes_and_the_command_to_run() {
    let ctx = day_of_events();
    let err = run_agenda(&ctx, agenda_args(&["--to", "today..week"])).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    let text = err.to_string();
    assert!(
        text.starts_with("--to cannot be \"today..week\"; it takes 2026-11-20,"),
        "{text}"
    );
    assert!(text.ends_with("; run dam agenda --to today"), "{text}");
}
