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
        let result = WORST_FIRST
            .iter()
            .map(|(outcome, ..)| *outcome)
            .find(|outcome| counted(*outcome) > 0)
            .unwrap_or(Outcome::NotRun);
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
        if self.rows.is_empty() {
            return (self.result == outcome)
                .then(|| self.line())
                .into_iter()
                .collect();
        }
        let name = self.name.as_str();
        self.rows
            .iter()
            .filter(|row| row.result == outcome)
            .map(|row| format!("{name} for {}: {}", row.name, row.detail))
            .collect()
    }
}
