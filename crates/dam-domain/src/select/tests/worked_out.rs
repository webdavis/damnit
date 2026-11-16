use std::collections::BTreeMap;

use super::fake::Fake;
use super::objects::{labelled, oid, task};
use crate::select::{Condition, WorkedOut, worked_out};
use crate::{Categories, Category, Event, Object, When};

fn fake() -> Fake {
    let mut fake = Fake::at("2026-11-20T12:00[UTC]");
    fake.categories = Categories::new(vec![
        Category {
            name: "effort".into(),
            values: vec!["deep".into(), "quick".into()],
            exclusive: true,
        },
        Category {
            name: "area".into(),
            values: vec!["ui".into(), "db".into()],
            exclusive: false,
        },
    ])
    .unwrap();
    fake
}

fn selected(field: &str, value: &str, object: &Object, fake: &Fake) -> bool {
    Condition::read(field, value).unwrap().holds(object, fake)
}

#[test]
fn category_maps_each_group_held_to_its_labels_and_leaves_out_the_rest() {
    let object = labelled(Object::Task(task(1)), &["deep", "ui", "db", "home"]);
    let got = worked_out(&object, &fake()).category;
    let want = BTreeMap::from([
        ("area".to_string(), vec!["db".to_string(), "ui".to_string()]),
        ("effort".to_string(), vec!["deep".to_string()]),
    ]);
    assert_eq!(got, want);
}

#[test]
fn overdue_and_blocked_print_what_the_filter_selects() {
    let mut fake = fake();
    let mut late = task(1);
    late.due = Some(When::Day(jiff::civil::date(2026, 11, 19)));
    late.base.depends = vec![oid(9)];
    fake.open_tasks.insert(oid(9));
    let late = Object::Task(late);
    let on_time = Object::Task(task(2));
    for object in [&late, &on_time] {
        let got = worked_out(object, &fake);
        assert_eq!(
            got.overdue,
            Some(selected("overdue", "true", object, &fake))
        );
        assert_eq!(
            got.blocked,
            Some(selected("blocked", "true", object, &fake))
        );
    }
    assert_eq!(worked_out(&late, &fake).overdue, Some(true));
    assert_eq!(worked_out(&late, &fake).blocked, Some(true));
    assert_eq!(worked_out(&on_time, &fake).blocked, Some(false));
}

#[test]
fn an_event_has_a_category_and_neither_overdue_nor_blocked() {
    let day = When::Day(jiff::civil::date(2026, 11, 1));
    let event = labelled(
        Object::Event(Event::new(oid(3), "standup", day.clone(), day)),
        &["ui"],
    );
    let got = worked_out(&event, &fake());
    assert_eq!(
        got,
        WorkedOut {
            category: BTreeMap::from([("area".to_string(), vec!["ui".to_string()])]),
            overdue: None,
            blocked: None,
        }
    );
}
