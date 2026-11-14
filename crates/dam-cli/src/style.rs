const ACCENT: &str = "\x1b[1;38;5;212m";
const ACCENT_QUIET: &str = "\x1b[38;5;212m";
const FAINT: &str = "\x1b[38;5;244m";
const RESET: &str = "\x1b[0m";

const GAP: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Paint {
    Color,
    Plain,
}

impl Paint {
    fn wrap(self, color: &str, text: &str) -> String {
        match self {
            Paint::Color => format!("{color}{text}{RESET}"),
            Paint::Plain => text.to_string(),
        }
    }
}

pub(crate) fn heading(paint: Paint, title: &str, blurb: &str) -> String {
    format!(
        "{} {} {}",
        paint.wrap(ACCENT, title),
        paint.wrap(ACCENT_QUIET, "──"),
        paint.wrap(FAINT, blurb)
    )
}

pub(crate) fn column<'a>(names: impl IntoIterator<Item = &'a str>) -> usize {
    names
        .into_iter()
        .map(|name| name.chars().count())
        .max()
        .unwrap_or(0)
        + GAP
}

pub(crate) fn field(paint: Paint, column: usize, name: &str, value: &str) -> String {
    let padding = " ".repeat(column.saturating_sub(name.chars().count()));
    format!("  {name}{padding}{}", paint.wrap(FAINT, value))
}

#[cfg(test)]
mod tests;
