pub(super) fn line_start(text: &str, offset: usize) -> usize {
    text[..offset].rfind('\n').map_or(0, |newline| newline + 1)
}

pub(super) fn line_end(text: &str, offset: usize) -> usize {
    text[offset..]
        .find('\n')
        .map_or(text.len(), |newline| offset + newline)
}

pub(super) fn indent_of_line(text: &str, offset: usize) -> &str {
    let start = line_start(text, offset);
    let line = &text[start..line_end(text, start)];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

pub(super) fn starts_its_line(text: &str, offset: usize) -> bool {
    text[line_start(text, offset)..offset]
        .chars()
        .all(|c| c == ' ' || c == '\t')
}

pub(super) fn line_ending(text: &str) -> &'static str {
    match text.find('\n') {
        Some(newline) if text[..newline].ends_with('\r') => "\r\n",
        _ => "\n",
    }
}
