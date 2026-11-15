use crate::args::StatusArgs;
use crate::testing::context;
use dam_application::{MissingTask, Notice, RemoteName};
use dam_domain::Oid;
use serde_json::json;

const STATUS_SCHEMA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/schemas/status.schema.json"
));

fn notices_schema() -> serde_json::Value {
    let status: serde_json::Value = serde_json::from_str(STATUS_SCHEMA).unwrap();
    let mut notices = status["properties"]["notices"].clone();
    notices["$defs"] = json!({"id": status["$defs"]["id"]});
    notices
}

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(byte))
}

fn missing(byte: u8, subject: &str) -> MissingTask {
    MissingTask {
        oid: oid(byte),
        subject: subject.into(),
    }
}

fn status_noticing_milk_and_eggs() -> crate::output::Report {
    let ctx = context();
    ctx.store
        .add_notice(&Notice::MissingUpstream {
            remote: RemoteName("fake".into()),
            tasks: vec![missing(1, "milk"), missing(2, "eggs")],
        })
        .unwrap();
    super::run_status(&ctx, StatusArgs { full: false }).unwrap()
}

#[test]
fn tasks_a_remote_no_longer_lists_read_as_one_line_and_one_line_each() {
    let human = status_noticing_milk_and_eggs().human;
    let expected = format!(
        "fake no longer lists 2 open tasks; dam done <oid> if finished there, dam rm <oid> if deleted\n    {} \"milk\"\n    {} \"eggs\"",
        oid(1).short(),
        oid(2).short()
    );
    assert!(human.contains(&expected), "{human}");
}

#[test]
fn tasks_a_remote_no_longer_lists_answer_as_one_missing_upstream_notice() {
    let data = status_noticing_milk_and_eggs().data;
    assert!(
        jsonschema::is_valid(&notices_schema(), &data["notices"]),
        "{data}"
    );
    assert_eq!(
        data["notices"],
        json!([{"kind": "missing_upstream", "remote": "fake", "tasks": [
            {"oid": oid(1).to_string(), "subject": "milk"},
            {"oid": oid(2).to_string(), "subject": "eggs"}
        ]}])
    );
}
