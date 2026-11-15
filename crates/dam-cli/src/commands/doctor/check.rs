use serde::Serialize;

use crate::args::CheckName;
use crate::style::Mark;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Outcome {
    Ok,
    Warning,
    Issue,
}

impl Outcome {
    pub(super) fn mark(self) -> Mark {
        match self {
            Outcome::Ok => Mark::Good,
            Outcome::Warning => Mark::Warn,
            Outcome::Issue => Mark::Bad,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Check {
    pub(super) name: CheckName,
    pub(super) result: Outcome,
    pub(super) detail: String,
    pub(super) advice: Option<String>,
    #[serde(flatten)]
    pub(super) files: Option<ConfigFiles>,
}

#[derive(Serialize)]
pub(crate) struct ConfigFiles {
    pub(super) config: Option<String>,
    pub(super) secrets: Option<String>,
    pub(super) warnings: Vec<String>,
}

impl Check {
    pub(super) fn of(name: CheckName, result: Outcome, detail: impl Into<String>) -> Check {
        Check {
            name,
            result,
            detail: detail.into(),
            advice: None,
            files: None,
        }
    }

    pub(super) fn advising(self, advice: impl Into<String>) -> Check {
        Check {
            advice: Some(advice.into()),
            ..self
        }
    }

    pub(super) fn reading(self, files: ConfigFiles) -> Check {
        Check {
            files: Some(files),
            ..self
        }
    }

    pub(super) fn line(&self) -> String {
        format!("{}: {}", self.name.as_str(), self.detail)
    }
}
