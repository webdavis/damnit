use super::{Moment, Stage, line};
use crate::output::Format;

fn moment(done: usize, seconds: u64) -> Moment<'static> {
    Moment {
        name: "task_pulled",
        remote: Some("todoist"),
        doing: "running task_pulled",
        did: "ran task_pulled",
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
        r#"{"step":"task_pulled","stage":"finished","done":2000,"total":2000,"seconds":14,"remote":"todoist"}"#
    );
}

#[test]
fn a_record_under_toon_is_the_toon_form() {
    let record = line(&moment(3, 1), Stage::Started, Format::Toon);
    assert!(record.contains("step: task_pulled"), "{record}");
    assert!(record.contains("stage: started"), "{record}");
}

fn waiting(seconds: u64) -> Moment<'static> {
    Moment {
        name: "pull",
        remote: Some("todoist"),
        doing: "pulling from todoist",
        did: "pulled from todoist",
        unit: "",
        done: 0,
        total: None,
        seconds,
    }
}

#[test]
fn a_step_without_a_unit_says_its_own_words_and_the_time_only() {
    assert_eq!(
        line(&waiting(3), Stage::Running(0), Format::Human),
        "⠋ pulling from todoist  3s"
    );
    assert_eq!(
        line(&waiting(3), Stage::Finished, Format::Human),
        "✓ pulled from todoist  3s"
    );
}

#[test]
fn a_failed_step_is_crossed_and_keeps_the_words_of_the_wait() {
    assert_eq!(
        line(&waiting(2), Stage::Failed, Format::Human),
        "✗ pulling from todoist failed  2s"
    );
}

#[test]
fn a_failed_record_says_failed() {
    assert_eq!(
        line(&waiting(2), Stage::Failed, Format::Json),
        r#"{"step":"pull","stage":"failed","done":0,"total":null,"seconds":2,"remote":"todoist"}"#
    );
}

#[test]
fn a_record_under_toon_names_the_remote_apart_from_the_step() {
    let record = line(&waiting(1), Stage::Started, Format::Toon);
    assert!(record.contains("step: pull"), "{record}");
    assert!(record.contains("remote: todoist"), "{record}");
}

#[test]
fn a_step_without_a_remote_has_no_remote_in_its_record() {
    let hook = Moment {
        name: "task_new",
        remote: None,
        ..waiting(1)
    };
    assert_eq!(
        line(&hook, Stage::Finished, Format::Json),
        r#"{"step":"task_new","stage":"finished","done":0,"total":null,"seconds":1}"#
    );
}
