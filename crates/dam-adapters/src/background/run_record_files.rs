use std::path::PathBuf;

use dam_application::{LastRun, RunEnding, RunRecords, ScheduleError, label_of};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::files::write_when_changed;

pub struct RunRecordFiles {
    pub folder: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Record {
    at: Timestamp,
    ended: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    why: Option<String>,
}

impl RunRecordFiles {
    fn path(&self, remote: &str) -> PathBuf {
        self.folder
            .join(format!("{}.last-run.json", label_of(remote)))
    }
}

impl RunRecords for RunRecordFiles {
    fn record(&self, remote: &str, run: &LastRun) -> Result<(), ScheduleError> {
        let (ended, why) = match &run.ended {
            RunEnding::Synced => ("synced", None),
            RunEnding::NotDue => ("not due", None),
            RunEnding::Failed(why) => ("failed", Some(why.clone())),
        };
        let record = Record {
            at: run.at,
            ended: ended.into(),
            why,
        };
        let text = serde_json::to_string(&record).unwrap_or_default();
        write_when_changed(&self.path(remote), &text).map(|_| ())
    }

    fn last(&self, remote: &str) -> Result<Option<LastRun>, ScheduleError> {
        let path = self.path(remote);
        let unreadable =
            |e: String| ScheduleError::Failed(format!("could not read {}: {e}", path.display()));
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(unreadable(e.to_string())),
        };
        let record: Record = serde_json::from_str(&text).map_err(|e| unreadable(e.to_string()))?;
        let ended = match (record.ended.as_str(), record.why) {
            ("synced", _) => RunEnding::Synced,
            ("not due", _) => RunEnding::NotDue,
            (_, why) => RunEnding::Failed(why.unwrap_or_default()),
        };
        Ok(Some(LastRun {
            at: record.at,
            ended,
        }))
    }
}

#[cfg(test)]
mod tests;
