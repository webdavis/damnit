use serde::Serialize;

use super::check::{Check, Outcome};
use crate::style::{Mark, Paint, advice, heading, needs_attention, row};
use crate::view::{Human, View};

pub(crate) struct Heading {
    pub(crate) title: &'static str,
    pub(crate) blurb: &'static str,
}

#[derive(Serialize)]
pub(crate) struct CheckReport {
    version: &'static str,
    command: &'static str,
    #[serde(skip)]
    heading: Heading,
    #[serde(skip_serializing_if = "Option::is_none")]
    created: Option<Vec<String>>,
    checks: Vec<Check>,
    needs_attention: Vec<String>,
}

impl CheckReport {
    pub(crate) fn new(command: &'static str, heading: Heading, checks: Vec<Check>) -> CheckReport {
        let needs_attention = [Outcome::Issue, Outcome::Warning]
            .iter()
            .flat_map(|outcome| lines_of(&checks, *outcome))
            .collect();
        CheckReport {
            version: env!("CARGO_PKG_VERSION"),
            command,
            heading,
            created: None,
            checks,
            needs_attention,
        }
    }

    pub(crate) fn creating(self, created: Option<Vec<String>>) -> CheckReport {
        CheckReport { created, ..self }
    }

    pub(super) fn found_an_issue(&self) -> bool {
        self.checks.iter().any(|c| c.result == Outcome::Issue)
    }

    fn lines(&self, paint: Paint) -> Vec<String> {
        let mut lines = vec![
            format!("dam {}", self.version),
            String::new(),
            heading(paint, self.heading.title, self.heading.blurb),
        ];
        lines.extend(
            self.created
                .as_deref()
                .map_or_else(Vec::new, |paths| created(paint, paths)),
        );
        for check in &self.checks {
            lines.push(row(paint, check.result.mark(), 2, &check.line()));
            let warnings = check.files.iter().flat_map(|files| &files.warnings);
            lines.extend(warnings.map(|w| row(paint, Mark::Warn, 4, w)));
            lines.extend(check.advice.iter().map(|text| advice(paint, 4, text)));
            for each in &check.rows {
                let text = format!("{}: {}", each.name, each.detail);
                lines.push(row(paint, each.result.mark(), 4, &text));
                lines.extend(each.advice.iter().map(|text| advice(paint, 6, text)));
            }
        }
        lines.extend(needs_attention(
            paint,
            &lines_of(&self.checks, Outcome::Issue),
            &lines_of(&self.checks, Outcome::Warning),
        ));
        lines
    }
}

fn created(paint: Paint, paths: &[String]) -> Vec<String> {
    if paths.is_empty() {
        return vec![row(paint, Mark::Quiet, 2, "created nothing")];
    }
    let each = |path| row(paint, Mark::Good, 2, &format!("created {path}"));
    paths.iter().map(each).collect()
}

fn lines_of(checks: &[Check], outcome: Outcome) -> Vec<String> {
    checks
        .iter()
        .flat_map(|check| check.attention(outcome))
        .collect()
}

impl View for CheckReport {
    fn human(&self) -> Human {
        Human::styled(|paint| self.lines(paint))
    }
}
