use crate::testing::{context, list_args};
use dam_domain::{Object, Oid, Task, When};
use jiff::civil::date;
use serde_json::Value;

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn trip_and_its_child() -> crate::context::Context {
    let ctx = context();
    let mut trip = Task::new(oid(1), "trip");
    trip.due = Some(When::Day(date(2026, 9, 21)));
    trip.deadline = Some(date(2026, 9, 16));
    ctx.store.put(&Object::Task(trip)).unwrap();
    let mut pack = Task::new(oid(2), "pack");
    pack.base.parent = Some(oid(1));
    ctx.store.put(&Object::Task(pack)).unwrap();
    ctx
}

fn trip_row(words: &[&str]) -> Value {
    let ctx = trip_and_its_child();
    let report = super::run_list(&ctx, list_args(words)).unwrap();
    let rows = report.data["objects"].as_array().unwrap().clone();
    rows.into_iter()
        .find(|row| row["subject"] == "trip")
        .unwrap()
}

#[test]
fn a_row_carries_its_days_until_due_and_deadline_and_its_open_children() {
    let trip = trip_row(&[]);
    assert_eq!(trip["days_until_due"], 3, "{trip}");
    assert_eq!(trip["days_until_deadline"], -2, "{trip}");
    assert_eq!(trip["children_open"], 1, "{trip}");
}

#[test]
fn a_filter_that_hides_the_child_leaves_the_count() {
    let trip = trip_row(&["--subject", "trip"]);
    assert_eq!(trip["children_open"], 1, "{trip}");
}

#[test]
fn the_worked_out_counts_cannot_be_sorted_by() {
    let ctx = trip_and_its_child();
    for field in ["days_until_due", "days_until_deadline", "children_open"] {
        assert!(
            super::run_list(&ctx, list_args(&["--sort", field])).is_err(),
            "{field}"
        );
    }
}
