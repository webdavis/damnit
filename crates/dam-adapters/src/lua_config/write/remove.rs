use std::ops::Range;

use super::lines::{line_end, line_start, starts_its_line};
use super::locate::TableLayout;

pub(super) fn remove(text: &str, table: &TableLayout, index: usize) -> String {
    let entry = &table.entries[index];
    let end = entry.separator.as_ref().map_or(entry.span.end, |s| s.end);
    let rest_of_line = &text[end..line_end(text, end)];
    let after = rest_of_line.trim();
    let blank_after = rest_of_line.len() - rest_of_line.trim_start_matches([' ', '\t']).len();
    let entry_cut = if starts_its_line(text, entry.span.start)
        && (after.is_empty() || is_line_comment(after))
    {
        line_start(text, entry.span.start)..(line_end(text, end) + 1).min(text.len())
    } else if after.is_empty() {
        text[..entry.span.start].trim_end_matches([' ', '\t']).len()..end + blank_after
    } else {
        entry.span.start..end + blank_after
    };
    let separator_before = match (&entry.separator, index.checked_sub(1)) {
        (None, Some(before)) => table.entries[before].separator.clone(),
        _ => None,
    };
    let text = cut(text, entry_cut);
    match separator_before {
        Some(separator) => cut(&text, separator),
        None => text,
    }
}

fn is_line_comment(text: &str) -> bool {
    text.strip_prefix("--").is_some_and(|comment| {
        !comment
            .strip_prefix('[')
            .is_some_and(|bracket| bracket.trim_start_matches('=').starts_with('['))
    })
}

fn cut(text: &str, range: Range<usize>) -> String {
    format!("{}{}", &text[..range.start], &text[range.end..])
}
