use super::lines::{indent_of_line, line_ending, line_start, starts_its_line};
use super::locate::TableLayout;

pub(in crate::lua_config) struct Indent<'a> {
    pub(super) leading: String,
    pub(super) per_level: &'a str,
}

impl<'a> Indent<'a> {
    pub(super) fn deeper(&self) -> Indent<'a> {
        Indent {
            leading: format!("{}{}", self.leading, self.per_level),
            per_level: self.per_level,
        }
    }
}

pub(super) fn insert(
    text: &str,
    table: &TableLayout,
    per_level: &str,
    entry: impl FnOnce(&Indent) -> String,
) -> String {
    let ending = line_ending(text);
    let entry = |indent: &Indent| entry(indent).replace('\n', ending);
    let last = table.entries.last();
    let indent = |leading: &str| Indent {
        leading: leading.to_string(),
        per_level,
    };
    if starts_its_line(text, table.close) {
        let inner = match last {
            Some(e) if starts_its_line(text, e.span.start) => {
                indent(indent_of_line(text, e.span.start))
            }
            _ => indent(indent_of_line(text, table.close)).deeper(),
        };
        let separator = if last.is_some_and(|e| e.separator.is_none()) {
            ""
        } else {
            ","
        };
        let line = format!("{}{}{separator}{ending}", inner.leading, entry(&inner));
        let text = splice(text, line_start(text, table.close), &line);
        return match last {
            Some(e) if e.separator.is_none() => splice(&text, e.span.end, ","),
            _ => text,
        };
    }
    let inner = indent(indent_of_line(text, table.open)).deeper();
    match last {
        Some(e) => match &e.separator {
            Some(separator) => splice(text, separator.end, &format!(" {},", entry(&inner))),
            None => splice(text, e.span.end, &format!(", {}", entry(&inner))),
        },
        None => {
            let inside = &text[table.open..table.close];
            let blank_end = table.open + inside.trim_end_matches([' ', '\t']).len();
            let outer = indent_of_line(text, table.open);
            let filled = format!("{ending}{}{},{ending}{outer}", inner.leading, entry(&inner));
            format!("{}{filled}{}", &text[..blank_end], &text[table.close..])
        }
    }
}

fn splice(text: &str, offset: usize, inserted: &str) -> String {
    format!("{}{inserted}{}", &text[..offset], &text[offset..])
}
