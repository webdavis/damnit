use super::tests::{args, oid};
use super::*;
use crate::testing::{ScriptedEditor, context};
use dam_domain::{Object, Task};

#[test]
fn a_line_typed_inside_a_multi_line_body_survives_the_reopen_after_a_refused_save() {
    let mut ctx = context();
    ctx.store
        .put(&Object::Task(Task::new(oid(1), "milk")))
        .unwrap();
    ctx.editor = Some(Box::new(ScriptedEditor::saving(vec![
        Box::new(|text| {
            text.replace(
                "body = \"\"",
                "body = \"\"\"line one\n# error: typed by me\nline three\"\"\"",
            )
            .replace("priority = 4", "priority = \"high\"")
        }),
        Box::new(|text| text.replace("priority = \"high\"", "priority = 2")),
    ])));
    run(
        &ctx,
        EditArgs {
            editor: true,
            ..args(&oid(1))
        },
    )
    .unwrap();
    assert_eq!(
        ctx.store.get(&oid(1)).unwrap().unwrap().base().body,
        "line one\n# error: typed by me\nline three"
    );
}
