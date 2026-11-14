use super::*;

#[test]
fn a_plain_heading_names_its_title_and_blurb() {
    assert_eq!(heading(Paint::Plain, "Files", "where"), "Files ── where");
}

#[test]
fn a_colored_heading_reads_the_same_once_its_escapes_are_gone() {
    let painted = heading(Paint::Color, "Files", "where");
    assert!(painted.contains('\x1b'));
    assert_eq!(without_escapes(&painted), "Files ── where");
}

#[test]
fn fields_line_their_values_up_after_the_longest_name() {
    let column = column(["owner", "done.interactive"]);
    assert_eq!(
        field(Paint::Plain, column, "owner", "Ada"),
        "  owner             Ada"
    );
    assert_eq!(
        field(Paint::Plain, column, "done.interactive", "false"),
        "  done.interactive  false"
    );
}

#[test]
fn a_colored_field_keeps_its_plain_layout() {
    let painted = field(Paint::Color, 9, "config", "/x/config.lua");
    assert_eq!(without_escapes(&painted), "  config   /x/config.lua");
}

fn without_escapes(text: &str) -> String {
    let mut plain = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => {
                chars.by_ref().find(|c| *c == 'm');
            }
            _ => plain.push(c),
        }
    }
    plain
}
