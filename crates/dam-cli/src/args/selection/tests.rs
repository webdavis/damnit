use clap::{CommandFactory, Parser};
use dam_domain::{FilterTable, TableValue, select_fields};

use crate::args::{Cli, Command};

fn typed(words: &[&str]) -> FilterTable {
    let line = ["dam", "list"].iter().chain(words);
    match Cli::try_parse_from(line).unwrap().command {
        Command::List(a) => a.selection.typed,
        _ => panic!("not list"),
    }
}

fn one(text: &str) -> TableValue {
    TableValue::One(text.into())
}

#[test]
fn every_row_of_the_field_table_is_a_flag_of_list_with_its_help() {
    let mut root = Cli::command();
    root.build();
    let list = root.find_subcommand("list").unwrap();
    for field in select_fields() {
        let flag = field.key.replace(['_', '.'], "-");
        let arg = list
            .get_arguments()
            .find(|a| a.get_long() == Some(flag.as_str()))
            .unwrap_or_else(|| panic!("--{flag} is missing"));
        assert!(arg.get_help().is_some(), "--{flag} has no help");
        assert!(arg.get_help_heading().is_some(), "--{flag} has no heading");
    }
}

#[test]
fn each_flag_repeats_and_keeps_every_value_in_order() {
    let t = typed(&["--labels", "bug", "--labels", "not:ui", "--priority", "1,2"]);
    let labels = TableValue::Many(vec!["bug".into(), "not:ui".into()]);
    assert_eq!(t.fields["labels"], labels);
    assert_eq!(t.fields["priority"], one("1,2"));
}

#[test]
fn a_bare_yes_no_flag_means_true_and_takes_a_value() {
    let t = typed(&["--done", "--blocked", "false", "--overdue"]);
    assert_eq!(t.fields["done"], one("true"));
    assert_eq!(t.fields["blocked"], one("false"));
    assert_eq!(t.fields["overdue"], one("true"));
}

#[test]
fn a_flag_with_a_dash_fills_the_key_it_was_made_from() {
    let t = typed(&["--focus-day", "today", "--completed-at", "week"]);
    assert_eq!(t.fields["focus.day"], one("today"));
    assert_eq!(t.fields["completed_at"], one("week"));
}

#[test]
fn any_splits_at_the_first_equals_and_reads_either_spelling_of_a_field() {
    let t = typed(&[
        "--any",
        "category=effort=deep",
        "--any",
        "completed-at=today",
        "--any",
        "focus.day=..today",
    ]);
    let keys: Vec<(&String, &TableValue)> = t.any.iter().flat_map(|a| &a.fields).collect();
    assert_eq!(
        keys,
        [
            (&"category".to_string(), &one("effort=deep")),
            (&"completed_at".to_string(), &one("today")),
            (&"focus.day".to_string(), &one("..today")),
        ]
    );
}

#[test]
fn an_unknown_field_in_any_is_kept_as_typed_for_the_filter_to_name() {
    let t = typed(&["--any", "lables=x", "--any", "overdue"]);
    assert!(t.any[0].fields.contains_key("lables"));
    assert_eq!(t.any[1].fields["overdue"], one(""));
}

#[test]
fn no_flag_types_nothing() {
    assert_eq!(typed(&[]), FilterTable::default());
    assert_eq!(typed(&["--global", "-o", "json"]), FilterTable::default());
}

#[test]
fn ls_is_gone() {
    assert!(Cli::try_parse_from(["dam", "ls"]).is_err());
    assert!(Cli::try_parse_from(["dam", "list", "due:today"]).is_err());
}
