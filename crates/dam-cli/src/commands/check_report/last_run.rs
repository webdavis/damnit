use std::path::Path;

use dam_domain::Timestamp;
use jiff::SignedDuration;
use serde::Serialize;

use super::check::{Check, Outcome};
use super::row::worst;
use crate::layout::ago_to_one_unit_and_never_in_the_future;
use crate::view::optional_display;

const STOPPED_AFTER: SignedDuration = SignedDuration::from_mins(10);
const REINSTALL: &str = "run dam sync disable, then dam sync enable";

#[derive(Serialize)]
pub(crate) struct LastRun {
    path: String,
    #[serde(serialize_with = "optional_display")]
    written: Option<Timestamp>,
    pub(super) result: Outcome,
    pub(super) detail: String,
    pub(super) advice: Option<String>,
}

impl LastRun {
    pub(crate) fn of(path: &Path, now: Timestamp, installed: bool) -> LastRun {
        let modified = std::fs::metadata(path).and_then(|meta| meta.modified());
        let written = modified.ok().and_then(|at| Timestamp::try_from(at).ok());
        let ago = |at| ago_to_one_unit_and_never_in_the_future(now, at);
        let said = |result, detail: String| (result, detail, None);
        let stopped = |since: String| {
            let detail = format!("the schedule is installed and {since}");
            (Outcome::Warning, detail, Some(REINSTALL.to_string()))
        };
        let (result, detail, advice) = match (installed, written) {
            (false, None) => said(Outcome::NotRun, "not written yet".to_string()),
            (false, Some(at)) => said(Outcome::NotRun, format!("written {}", ago(at))),
            (true, None) => stopped("has never run".to_string()),
            (true, Some(at)) if now.duration_since(at) > STOPPED_AFTER => {
                stopped(format!("has not run since {}", ago(at)))
            }
            (true, Some(at)) => said(Outcome::Ok, format!("written {}", ago(at))),
        };
        LastRun {
            path: path.display().to_string(),
            written,
            result,
            detail,
            advice,
        }
    }

    pub(super) fn line(&self) -> String {
        format!("last run {}, {}", self.path, self.detail)
    }
}

impl Check {
    pub(crate) fn naming(self, last_run: LastRun) -> Check {
        let result = match last_run.result {
            Outcome::Warning => worst(&[self.result, Outcome::Warning]),
            _ => self.result,
        };
        Check {
            result,
            last_run: Some(last_run),
            ..self
        }
    }
}
