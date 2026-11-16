use super::{Moment, Stage, line};
use crate::output::Format;

fn moment(done: usize, seconds: u64) -> Moment<'static> {
    Moment {
        name: "task_pulled",
        unit: "tasks",
        done,
        total: Some(2000),
        seconds,
    }
}

#[test]
fn a_running_line_spins_and_counts_with_the_time_spent() {
    assert_eq!(
        line(&moment(812, 14), Stage::Running(1), Format::Human),
        "⠙ running task_pulled  812 of 2,000 tasks  14s"
    );
}

#[test]
fn a_finished_line_is_ticked() {
    assert_eq!(
        line(&moment(2000, 14), Stage::Finished, Format::Human),
        "✓ ran task_pulled  2,000 of 2,000 tasks  14s"
    );
}

#[test]
fn a_started_record_in_plain_text_has_no_mark() {
    assert_eq!(
        line(&moment(0, 0), Stage::Started, Format::Human),
        "running task_pulled  0 of 2,000 tasks  0s"
    );
}

#[test]
fn a_finished_record_under_json_is_one_line_with_name_count_total_and_seconds() {
    assert_eq!(
        line(&moment(2000, 14), Stage::Finished, Format::Json),
        r#"{"step":"task_pulled","stage":"finished","done":2000,"total":2000,"seconds":14}"#
    );
}

#[test]
fn a_record_under_toon_is_the_toon_form() {
    let record = line(&moment(3, 1), Stage::Started, Format::Toon);
    assert!(record.contains("step: task_pulled"), "{record}");
    assert!(record.contains("stage: started"), "{record}");
}
