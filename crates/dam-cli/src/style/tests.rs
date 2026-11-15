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

#[test]
fn a_heading_without_a_blurb_is_its_title_alone() {
    assert_eq!(
        heading(Paint::Plain, "Needs attention", ""),
        "Needs attention"
    );
}

#[test]
fn each_mark_leads_its_row_with_a_symbol_that_carries_its_meaning() {
    let rows: Vec<String> = [Mark::Good, Mark::Warn, Mark::Bad]
        .into_iter()
        .map(|mark| row(Paint::Plain, mark, 2, "config: read"))
        .collect();
    assert_eq!(
        rows,
        ["  ✓ config: read", "  ⚠ config: read", "  ✗ config: read"]
    );
    assert_eq!(row(Paint::Plain, Mark::Warn, 4, "token"), "    ⚠ token");
}

#[test]
fn a_colored_row_tints_its_mark_alone() {
    let painted = row(Paint::Color, Mark::Bad, 2, "database: version 13");
    assert!(painted.starts_with("  \x1b["), "{painted:?}");
    assert!(
        painted.ends_with("\x1b[0m database: version 13"),
        "{painted:?}"
    );
    assert_eq!(without_escapes(&painted), "  ✗ database: version 13");
}

#[test]
fn advice_follows_an_arrow_under_its_row() {
    assert_eq!(advice(Paint::Plain, "run dam push"), "    → run dam push");
    assert_eq!(
        without_escapes(&advice(Paint::Color, "run dam push")),
        "    → run dam push"
    );
}

#[test]
fn nothing_to_attend_to_closes_with_a_good_mark() {
    assert_eq!(
        needs_attention(Paint::Plain, &[], &[]),
        ["", "Needs attention", "  ✓ nothing needs attention"]
    );
}

#[test]
fn issues_come_before_warnings_counted_then_numbered() {
    let issues = ["helpers: dam-remote-todoist is not installed".to_string()];
    let warnings = [
        "pushes: 2 changes still owed to todoist".to_string(),
        "database: version 11".to_string(),
    ];
    assert_eq!(
        needs_attention(Paint::Plain, &issues, &warnings),
        [
            "",
            "Needs attention",
            "  ✗ 1 issue to fix, 2 warnings to look at",
            "  1. helpers: dam-remote-todoist is not installed",
            "  2. pushes: 2 changes still owed to todoist",
            "  3. database: version 11",
        ]
    );
}

#[test]
fn warnings_alone_close_with_a_warning_mark() {
    let warnings = ["pushes: 1 change still owed to todoist".to_string()];
    assert_eq!(
        needs_attention(Paint::Plain, &[], &warnings)[2],
        "  ⚠ 1 warning to look at"
    );
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
