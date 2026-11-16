use std::ffi::OsString;

use dam_domain::TableValue;
use serde_json::json;
use tempfile::NamedTempFile;

use crate::args::{AgendaArgs, Command};
use crate::request::parse;

fn requested(request: serde_json::Value) -> Result<AgendaArgs, String> {
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), request.to_string()).unwrap();
    let line = ["dam", "agenda", "--request"]
        .map(OsString::from)
        .into_iter()
        .chain([file.path().into()])
        .collect();
    match parse(line)
        .map_err(|refused| refused.error.to_string())?
        .command
    {
        Command::Agenda(args) => Ok(args),
        other => panic!("not agenda: {other:?}"),
    }
}

#[test]
fn an_agenda_request_reads_the_selection_and_the_window() {
    let args = requested(json!({
        "labels": "work",
        "any": [{"response": "accepted"}, {"organizer.email": "a@b"}],
        "to": "today",
    }))
    .unwrap();
    assert_eq!(
        args.selection.typed.fields["labels"],
        TableValue::One("work".into())
    );
    assert_eq!(args.selection.typed.any.len(), 2);
    assert_eq!(args.to.as_deref(), Some("today"));
}

#[test]
fn an_agenda_request_names_agenda_and_takes_no_order() {
    let unknown = requested(json!({"duee": "today"})).unwrap_err();
    assert_eq!(
        unknown,
        "--request: duee is not a key dam agenda takes; the closest is due"
    );
    let sort = requested(json!({"sort": "start"})).unwrap_err();
    assert_eq!(sort, "--request: sort is not a key dam agenda takes");
}
