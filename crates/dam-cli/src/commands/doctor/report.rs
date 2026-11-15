use serde::Serialize;

use super::check::{Check, Outcome};
use crate::style::{Mark, Paint, advice, heading, needs_attention, row};
use crate::view::{Human, View};

const SECTION: &str = "Setup";
const BLURB: &str = "whether this machine is set up to run dam";

#[derive(Serialize)]
pub(super) struct Doctor {
    version: &'static str,
    command: &'static str,
    checks: Vec<Check>,
    needs_attention: Vec<String>,
}

impl Doctor {
    pub(super) fn new(command: &'static str, checks: Vec<Check>) -> Doctor {
        let needs_attention = [Outcome::Issue, Outcome::Warning]
            .iter()
            .flat_map(|outcome| lines_of(&checks, *outcome))
            .collect();
        Doctor {
            version: env!("CARGO_PKG_VERSION"),
            command,
            checks,
            needs_attention,
        }
    }

    pub(super) fn found_an_issue(&self) -> bool {
        self.checks.iter().any(|c| c.result == Outcome::Issue)
    }

    fn lines(&self, paint: Paint) -> Vec<String> {
        let mut lines = vec![
            format!("dam {}", self.version),
            String::new(),
            heading(paint, SECTION, BLURB),
        ];
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

fn lines_of(checks: &[Check], outcome: Outcome) -> Vec<String> {
    checks
        .iter()
        .flat_map(|check| check.attention(outcome))
        .collect()
}

impl View for Doctor {
    fn human(&self) -> Human {
        Human::styled(|paint| self.lines(paint))
    }
}
