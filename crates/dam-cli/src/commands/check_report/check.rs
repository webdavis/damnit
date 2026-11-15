use serde::Serialize;

use super::last_run::LastRun;
use super::row::Row;
use crate::args::CheckName;
use crate::style::Mark;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    Ok,
    Warning,
    Issue,
    NotRun,
}

impl Outcome {
    pub(super) fn mark(self) -> Mark {
        match self {
            Outcome::Ok => Mark::Good,
            Outcome::Warning => Mark::Warn,
            Outcome::Issue => Mark::Bad,
            Outcome::NotRun => Mark::Quiet,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Check {
    pub(super) name: CheckName,
    pub(super) result: Outcome,
    pub(super) detail: String,
    pub(super) advice: Option<String>,
    pub(super) rows: Vec<Row>,
    #[serde(flatten)]
    pub(super) files: Option<ConfigFiles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_run: Option<LastRun>,
}

#[derive(Serialize)]
pub(crate) struct ConfigFiles {
    pub(crate) config: Option<String>,
    pub(crate) secrets: Option<String>,
    pub(crate) warnings: Vec<String>,
}

impl Check {
    pub(crate) fn of(name: CheckName, result: Outcome, detail: impl Into<String>) -> Check {
        Check {
            name,
            result,
            detail: detail.into(),
            advice: None,
            rows: Vec::new(),
            files: None,
            last_run: None,
        }
    }

    pub(crate) fn advising(self, advice: impl Into<String>) -> Check {
        Check {
            advice: Some(advice.into()),
            ..self
        }
    }

    pub(crate) fn reading(self, files: ConfigFiles) -> Check {
        Check {
            files: Some(files),
            ..self
        }
    }

    pub(super) fn line(&self) -> String {
        format!("{}: {}", self.name.as_str(), self.detail)
    }
}
