use super::*;
use dam_domain::{Object, Task};

fn pulled_task(remote_id: &str) -> serde_json::Value {
    let oid = Oid::generate(&mut |b: &mut [u8]| b.fill(1));
    let mut value =
        serde_json::to_value(to_wire(&Object::Task(Task::new(oid, "milk")), None)).unwrap();
    value["remote_id"] = remote_id.into();
    value
}

#[test]
fn an_object_whose_field_has_the_wrong_type_is_rejected_alone_and_the_rest_land() {
    let mut bad = pulled_task("r2");
    bad["task"]["priority"] = "high".into();
    let line = serde_json::json!({
        "objects": [pulled_task("r1"), bad, pulled_task("r3")],
        "removed": [],
        "sync": null
    })
    .to_string();
    let outcome = pull_from_wire(serde_json::from_str(&line).unwrap());
    let landed: Vec<_> = outcome
        .objects
        .iter()
        .map(|o| o.remote_id.as_str())
        .collect();
    assert_eq!(landed, vec!["r1", "r3"]);
    assert_eq!(
        outcome.rejected,
        vec![RejectedObject {
            remote_id: "r2".into(),
            why: "invalid type: string \"high\", expected u8".into(),
        }]
    );
}
