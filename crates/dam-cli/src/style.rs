mod attention;

pub(crate) use attention::needs_attention;

const ACCENT: &str = "\x1b[1;38;5;212m";
const ACCENT_QUIET: &str = "\x1b[38;5;212m";
const FAINT: &str = "\x1b[38;5;244m";
const GOOD: &str = "\x1b[38;5;42m";
const WARN: &str = "\x1b[38;5;221m";
const BAD: &str = "\x1b[1;38;5;203m";
const RESET: &str = "\x1b[0m";

const GAP: usize = 2;
const ADVICE_INDENT: usize = 4;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    Good,
    Warn,
    Bad,
}

impl Mark {
    fn symbol_and_color(self) -> (&'static str, &'static str) {
        match self {
            Mark::Good => ("✓", GOOD),
            Mark::Warn => ("⚠", WARN),
            Mark::Bad => ("✗", BAD),
        }
    }
}

pub(crate) fn heading(paint: Paint, title: &str, blurb: &str) -> String {
    if blurb.is_empty() {
        return paint.wrap(ACCENT, title);
    }
    format!(
        "{} {} {}",
        paint.wrap(ACCENT, title),
        paint.wrap(ACCENT_QUIET, "──"),
        paint.wrap(FAINT, blurb)
    )
}

pub(crate) fn row(paint: Paint, mark: Mark, indent: usize, text: &str) -> String {
    let (symbol, color) = mark.symbol_and_color();
    format!("{}{} {text}", " ".repeat(indent), paint.wrap(color, symbol))
}

pub(crate) fn advice(paint: Paint, text: &str) -> String {
    format!(
        "{}{} {text}",
        " ".repeat(ADVICE_INDENT),
        paint.wrap(FAINT, "→")
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
