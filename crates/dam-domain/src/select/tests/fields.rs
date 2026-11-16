use std::collections::BTreeSet;

use crate::select::{FieldType, Has, Holds, closest_field, select_field, select_fields};

#[test]
fn every_field_of_the_design_has_one_row() {
    let keys: Vec<&str> = select_fields().iter().map(|f| f.key).collect();
    let unique: BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(keys.len(), unique.len());
    for key in [
        "kind",
        "subject",
        "body",
        "path",
        "labels",
        "category",
        "parent",
        "depends",
        "reminders",
        "filter",
        "done",
        "overdue",
        "blocked",
        "priority",
        "due",
        "deadline",
        "focus.day",
        "completed_at",
        "estimate",
        "attached",
        "start",
        "end",
        "location",
        "status",
        "transparency",
        "visibility",
        "event_type",
        "response",
        "organizer.email",
        "attendees.email",
    ] {
        assert!(unique.contains(key), "{key} has no row");
    }
}

#[test]
fn a_row_says_its_type_and_which_kind_has_it() {
    let due = select_field("due").unwrap();
    assert!(matches!(due.ty, FieldType::Date));
    assert_eq!(due.has, Has::Task);
    assert_eq!(due.holds, Holds::OneOrNone);
    let start = select_field("start").unwrap();
    assert_eq!((start.has, start.holds), (Has::Event, Holds::One));
    assert_eq!(select_field("labels").unwrap().holds, Holds::Many);
    assert_eq!(select_field("subject").unwrap().has, Has::Both);
    assert!(select_field("any").is_none());
}

#[test]
fn a_choice_row_lists_its_choices_as_its_shapes() {
    let shapes = select_field("transparency").unwrap().shapes();
    assert!(shapes.contains(&"busy".to_string()) && shapes.contains(&"free".to_string()));
    let response = select_field("response").unwrap().shapes();
    assert!(response.contains(&"needs_action".to_string()));
    assert!(response.contains(&"none".to_string()));
    assert!(
        !select_field("kind")
            .unwrap()
            .shapes()
            .contains(&"none".to_string())
    );
}

#[test]
fn a_mistyped_field_names_the_closest_one() {
    assert_eq!(closest_field("lables"), Some("labels"));
    assert_eq!(closest_field("completed-at"), Some("completed_at"));
    assert_eq!(closest_field("prority"), Some("priority"));
    assert_eq!(closest_field("xyzzyplugh"), None);
}
