use super::{Mark, Paint, heading, row};

const TITLE: &str = "Needs attention";
const NOTHING: &str = "nothing needs attention";

pub(crate) fn needs_attention(paint: Paint, issues: &[String], warnings: &[String]) -> Vec<String> {
    let mut lines = vec![String::new(), heading(paint, TITLE, "")];
    let mark = match (issues.is_empty(), warnings.is_empty()) {
        (true, true) => {
            lines.push(row(paint, Mark::Good, 2, NOTHING));
            return lines;
        }
        (true, false) => Mark::Warn,
        (false, _) => Mark::Bad,
    };
    let headline: Vec<String> = counted(issues.len(), "issue", "to fix")
        .into_iter()
        .chain(counted(warnings.len(), "warning", "to look at"))
        .collect();
    lines.push(row(paint, mark, 2, &headline.join(", ")));
    lines.extend(
        issues
            .iter()
            .chain(warnings)
            .enumerate()
            .map(|(index, entry)| format!("  {}. {entry}", index + 1)),
    );
    lines
}

fn counted(count: usize, noun: &str, purpose: &str) -> Option<String> {
    match count {
        0 => None,
        1 => Some(format!("1 {noun} {purpose}")),
        _ => Some(format!("{count} {noun}s {purpose}")),
    }
}
