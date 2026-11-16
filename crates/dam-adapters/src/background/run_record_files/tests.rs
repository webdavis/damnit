use dam_application::{LastRun, RunEnding, RunRecords};
use jiff::Timestamp;

use super::RunRecordFiles;

fn run(ended: RunEnding) -> LastRun {
    LastRun {
        at: Timestamp::from_second(1_800_000_000).unwrap(),
        ended,
    }
}

#[test]
fn each_ending_is_read_back_as_written_and_the_newest_replaces_the_last() {
    let state = tempfile::tempdir().unwrap();
    let records = RunRecordFiles {
        folder: state.path().join("background"),
    };
    assert_eq!(records.last("todoist"), Ok(None));
    for ended in [
        RunEnding::Synced,
        RunEnding::NotDue,
        RunEnding::Failed("the helper is missing".into()),
    ] {
        records.record("todoist", &run(ended.clone())).unwrap();
        assert_eq!(records.last("todoist"), Ok(Some(run(ended))));
    }
    assert_eq!(records.last("gcal"), Ok(None));
}

#[test]
fn an_unreadable_record_is_reported() {
    let state = tempfile::tempdir().unwrap();
    std::fs::write(state.path().join("dam-sync.todoist.last-run.json"), "{").unwrap();
    let records = RunRecordFiles {
        folder: state.path().to_path_buf(),
    };
    assert!(records.last("todoist").is_err());
}
