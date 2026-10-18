use super::tests::{args, oid};
use super::*;
use crate::args::EditArgs;
use crate::testing::{Save, ScriptedEditor, context, machine_context};
use dam_domain::{Object, Task};

#[test]
fn the_editor_round_trip_applies_the_saved_text() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let saved = dam_adapters::render_template(&ctx.store.get(&oid(1)).unwrap().unwrap())
        .replace("subject = \"milk\"", "subject = \"eggs\"");
    ctx.editor = Some(Box::new(ScriptedEditor::saving(vec![Box::new(
        move |_| saved.clone(),
    )])));
    run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap();
    assert_eq!(
        ctx.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "eggs"
    );
}

#[test]
fn an_unchanged_editor_save_is_a_cancel() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    ctx.editor = Some(Box::new(ScriptedEditor::saving_unchanged()));
    assert!(matches!(
        run(
            &mut ctx,
            EditArgs {
                editor: true,
                ..args(&oid(1))
            }
        ),
        Err(CliError::Cancelled)
    ));
}

#[test]
fn a_save_that_never_parses_stops_after_it_comes_back_unchanged() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    ctx.editor = Some(Box::new(ScriptedEditor::saving(vec![
        Box::new(|_| "priority = \"high\"\n".into()),
        Box::new(|_| "priority = \"high\"\n".into()),
    ])));
    let err = run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("priority"), "{err}");
    assert_eq!(
        ctx.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "milk"
    );
}

#[test]
fn a_machine_format_refuses_to_open_an_editor() {
    let mut ctx = machine_context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let err = run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert_eq!(err.to_string(), Refusal::NeedsAnEditor.to_string());
    assert_eq!(err.exit_code(), 4);
}

#[test]
fn a_bad_save_kept_as_it_was_reopened_reports_the_parse_error_not_a_cancel() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    ctx.editor = Some(Box::new(ScriptedEditor::saving(vec![Box::new(|_| {
        "priority = \"high\"\n".into()
    })])));
    let err = run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 2, "{err}");
    assert!(err.to_string().contains("priority"), "{err}");
}

#[test]
fn a_second_save_that_still_does_not_parse_is_refused_instead_of_reopened_forever() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    let opened = std::rc::Rc::new(std::cell::Cell::new(0));
    let saves = (0..5)
        .map(|_| {
            let opened = opened.clone();
            Box::new(move |text: &str| {
                opened.set(opened.get() + 1);
                format!("{text}priority = \"high\"\n")
            }) as Save
        })
        .collect();
    ctx.editor = Some(Box::new(ScriptedEditor::saving(saves)));
    let err = run(
        &mut ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 2, "{err}");
    assert!(err.to_string().contains("duplicate key"), "{err}");
    assert_eq!(opened.get(), 2, "the editor reopened more than once");
}
