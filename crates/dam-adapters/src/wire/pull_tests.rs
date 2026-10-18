use super::*;
use dam_domain::{Object, Task};

fn pulled_task(remote_id: &str) -> serde_json::Value {
    let oid = Oid::generate(&mut |b: &mut [u8]| b.fill(1));
    let mut value =
        serde_json::to_value(to_wire(&Object::Task(Task::new(oid, "milk")), None)).unwrap();
    value["remote_id"] = remote_id.into();
    value
}

fn pull(objects: Vec<serde_json::Value>) -> PullOutcome {
    let line = serde_json::json!({ "objects": objects, "removed": [], "sync": null }).to_string();
    pull_from_wire(serde_json::from_str(&line).unwrap())
}

#[test]
fn an_object_whose_field_has_the_wrong_type_is_rejected_alone_and_the_rest_land() {
    let mut bad = pulled_task("r2");
    bad["task"]["priority"] = "high".into();
    let outcome = pull(vec![pulled_task("r1"), bad, pulled_task("r3")]);
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
            why: "priority: cannot read \"high\"".into(),
        }]
    );
    assert!(!outcome.rejected[0].why.contains("invalid type"));
}

#[test]
fn an_unreadable_object_with_no_remote_id_is_skipped_as_nameless() {
    let mut bad = pulled_task("r2");
    bad["task"]["priority"] = "high".into();
    bad.as_object_mut().unwrap().remove("remote_id");
    let outcome = pull(vec![bad]);
    assert!(outcome.objects.is_empty());
    assert_eq!(
        outcome.rejected,
        vec![RejectedObject {
            remote_id: String::new(),
            why: "an object with no remote id was skipped".into(),
        }]
    );
    assert!(!outcome.rejected[0].why.contains("invalid type"));
}

#[test]
fn an_object_missing_a_required_field_names_that_field() {
    let mut bad = pulled_task("r2");
    bad["task"].as_object_mut().unwrap().remove("done");
    let outcome = pull(vec![bad]);
    assert_eq!(outcome.rejected[0].why, "done: missing");
}
