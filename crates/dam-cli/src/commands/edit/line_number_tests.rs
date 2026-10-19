use super::tests::{args, oid};
use super::*;
use crate::testing::{Save, ScriptedEditor, context};
use dam_domain::{Object, Task};

fn refused_with_kept_file(saves: Vec<Save>) -> (String, Vec<String>) {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    ctx.editor = Some(Box::new(ScriptedEditor::saving(saves)));
    let message = run(
        &ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err()
    .to_string();
    let (refusal, kept) = message
        .split_once("; your edit is kept at ")
        .unwrap_or_else(|| panic!("no kept file named: {message}"));
    let lines = std::fs::read_to_string(kept)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    (refusal.to_string(), lines)
}

fn line_named_by(text: &str) -> &str {
    text.strip_prefix("line ")
        .and_then(|rest| rest.split([':', ',']).next())
        .unwrap_or_else(|| panic!("no line number in: {text}"))
}

fn line_of<'a>(lines: &'a [String], number: &str) -> &'a str {
    &lines[number.parse::<usize>().unwrap() - 1]
}

#[test]
fn an_unchanged_re_save_names_the_line_of_the_bad_value_in_the_kept_file() {
    let (refusal, kept) = refused_with_kept_file(vec![Box::new(|text| {
        text.replace("priority = 4", "priority = \"high\"")
    })]);
    assert_eq!(
        line_of(&kept, line_named_by(&refusal)),
        "priority = \"high\""
    );
}

#[test]
fn the_error_header_names_the_line_of_the_bad_value_below_it() {
    let (_, kept) = refused_with_kept_file(
        (0..2)
            .map(|_| Box::new(|text: &str| format!("{text}priority = \"high\"\n")) as Save)
            .collect(),
    );
    let header = kept[0].strip_prefix("# error: ").unwrap();
    assert_eq!(line_of(&kept, line_named_by(header)), "priority = \"high\"");
}
