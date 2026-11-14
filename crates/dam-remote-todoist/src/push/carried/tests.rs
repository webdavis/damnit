use super::*;
use dam_protocol::WireTask;

fn object(path: &str, done: bool) -> WireObject {
    WireObject {
        oid: String::new(),
        remote_id: None,
        kind: "task".into(),
        subject: "Now".into(),
        body: String::new(),
        path: path.into(),
        labels: vec![],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done,
            completed_at: None,
            priority: 4,
            due: None,
            deadline: None,
            event: None,
            estimate: None,
            focus: None,
        }),
        event: None,
    }
}

#[test]
fn names_every_field_the_container_cannot_carry_and_nothing_else() {
    let declared = ["subject".to_string(), "done".into(), "body".into()];
    let (object, held) = (object("Work/", true), object("Work/", false));
    assert_eq!(
        fields_the_container_cannot_carry(&PROJECT, &declared[..2], &object, Some(&held)),
        None
    );
    assert_eq!(
        fields_the_container_cannot_carry(&SECTION, &declared, &object, Some(&held)).as_deref(),
        Some("a Todoist section carries only its name; done did not change")
    );
}

#[test]
fn without_a_held_object_every_uncarried_field_counts_as_changed() {
    let declared = ["path".to_string(), "done".into()];
    let object = object("Work/", false);
    assert_eq!(
        fields_the_container_cannot_carry(&SECTION, &declared, &object, None).as_deref(),
        Some("a Todoist section carries only its name; path, done did not change")
    );
}
