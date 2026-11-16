use super::run_reminders;
use super::tests::reminded_task;
use crate::args::RemindersArgs;
use crate::testing::context;

fn window(from: &str, to: &str) -> RemindersArgs {
    RemindersArgs {
        scope: Default::default(),
        place: Default::default(),
        from: Some(from.into()),
        to: Some(to.into()),
    }
}

#[test]
fn a_day_as_to_counts_whole() {
    let ctx = context();
    ctx.store.put(&reminded_task()).unwrap();
    let due = run_reminders(&ctx, window("2026-01-01", "2026-01-01"))
        .unwrap()
        .data;
    let reminders = due["reminders"].as_array().unwrap();
    assert_eq!(reminders.len(), 1, "{due}");
    assert_eq!(reminders[0]["minutes_before"], 1440);
}

#[test]
fn a_bad_window_edge_names_the_shapes_and_reminders_in_the_command_to_run() {
    let err = run_reminders(&context(), window("today", "soon")).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    let text = err.to_string();
    assert!(
        text.starts_with("--to cannot be \"soon\"; it takes"),
        "{text}"
    );
    assert!(text.ends_with("; run dam reminders --to today"), "{text}");
}
