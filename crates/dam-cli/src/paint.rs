use std::io::IsTerminal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tint {
    Plain,
    Removed,
    Added,
}

pub(crate) fn wanted(no_color_flag: bool) -> bool {
    let set = |name| std::env::var_os(name).is_some_and(|v| !v.is_empty() && v != "0");
    !no_color_flag && !set("NO_COLOR") && (set("CLICOLOR_FORCE") || std::io::stdout().is_terminal())
}

pub(crate) fn painted(text: &str, tints: &[Tint]) -> String {
    text.split('\n')
        .zip(tints.iter().copied().chain(std::iter::repeat(Tint::Plain)))
        .map(|(line, tint)| match tint {
            Tint::Plain => line.to_string(),
            Tint::Removed => format!("\x1b[31m{line}\x1b[0m"),
            Tint::Added => format!("\x1b[32m{line}\x1b[0m"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
