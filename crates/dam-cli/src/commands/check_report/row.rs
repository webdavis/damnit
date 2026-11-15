use serde::Serialize;

use super::check::{Check, Outcome};
use crate::args::CheckName;

const WORST_FIRST: [(Outcome, &str, &str); 4] = [
    (Outcome::Issue, "issue", "issues"),
    (Outcome::Warning, "warning", "warnings"),
    (Outcome::Ok, "ok", "ok"),
    (Outcome::NotRun, "not checked", "not checked"),
];

#[derive(Serialize)]
pub(crate) struct Row {
    pub(super) name: String,
    pub(super) result: Outcome,
    pub(super) detail: String,
    pub(super) advice: Option<String>,
}

pub(super) fn worst(outcomes: &[Outcome]) -> Outcome {
    (WORST_FIRST.iter())
        .map(|(outcome, ..)| *outcome)
        .find(|outcome| outcomes.contains(outcome))
        .unwrap_or(Outcome::NotRun)
}

impl Row {
    pub(crate) fn of(name: &str, result: Outcome, detail: impl Into<String>) -> Row {
        Row {
            name: name.to_string(),
            result,
            detail: detail.into(),
            advice: None,
        }
    }

    pub(crate) fn advising(self, advice: impl Into<String>) -> Row {
        Row {
            advice: Some(advice.into()),
            ..self
        }
    }
}

impl Check {
    pub(crate) fn of_rows(name: CheckName, rows: Vec<Row>) -> Check {
        let counted = |outcome| rows.iter().filter(|row| row.result == outcome).count();
        let result = worst(&rows.iter().map(|row| row.result).collect::<Vec<_>>());
        let counts: Vec<String> = WORST_FIRST
            .iter()
            .filter_map(|(outcome, one, many)| match counted(*outcome) {
                0 => None,
                1 => Some(format!("1 {one}")),
                n => Some(format!("{n} {many}")),
            })
            .collect();
        let remotes = match rows.len() {
            1 => "1 remote".to_string(),
            n => format!("{n} remotes"),
        };
        let detail = format!("{remotes}: {}", counts.join(", "));
        Check {
            rows,
            ..Check::of(name, result, detail)
        }
    }

    pub(super) fn attention(&self, outcome: Outcome) -> Vec<String> {
        let speaks_for_itself = self.rows.is_empty() && self.last_run.is_none();
        let own = (speaks_for_itself && self.result == outcome).then(|| self.line());
        let name = self.name.as_str();
        let last_run = (self.last_run.iter())
            .filter(|run| run.result == outcome)
            .map(|run| format!("{name}: {}", run.detail));
        let rows = (self.rows.iter())
            .filter(|row| row.result == outcome)
            .map(|row| format!("{name} for {}: {}", row.name, row.detail));
        own.into_iter().chain(last_run).chain(rows).collect()
    }
}
