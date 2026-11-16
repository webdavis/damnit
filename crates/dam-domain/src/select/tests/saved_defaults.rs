use super::fake::Fake;
use super::objects::task;
use super::tables::{either, many, one, table};
use crate::Object;
use crate::select::{Filter, FilterTable};

fn saved(name: &str) -> Option<FilterTable> {
    match name {
        "today" => Some(table(&[("due", one("..today"))])),
        "allhands" => Some(table(&[
            ("kind", one("event")),
            ("subject", one("all hands")),
        ])),
        "soon" => Some(table(&[("filter", one("today")), ("path", one("work/"))])),
        _ => None,
    }
}

#[test]
fn one_saved_filter_lends_its_fields_to_the_defaults_as_if_typed() {
    let typed = table(&[("filter", one("today")), ("done", one("true"))]);
    assert_eq!(
        typed.with_saved(&saved),
        table(&[
            ("filter", one("today")),
            ("done", one("true")),
            ("due", one("..today"))
        ])
    );
}

#[test]
fn a_saved_field_beside_a_typed_one_repeats_the_flag() {
    let typed = table(&[("filter", one("today")), ("due", one("week"))]);
    let view = typed.with_saved(&saved);
    assert_eq!(view.fields["due"], many(&["week", "..today"]));
}

#[test]
fn either_filter_lends_its_fields_as_any_and_not_lends_nothing() {
    let typed = table(&[("filter", one("today,allhands"))]);
    assert_eq!(
        typed.with_saved(&saved),
        either(
            table(&[("filter", one("today,allhands"))]),
            vec![saved("today").unwrap(), saved("allhands").unwrap()]
        )
    );
    let not = table(&[("filter", one("not:today"))]);
    assert_eq!(not.with_saved(&saved), not);
}

#[test]
fn a_filter_reached_through_a_filter_lends_its_fields_too() {
    let view = table(&[("filter", one("soon"))]).with_saved(&saved);
    assert_eq!(view.fields["due"], one("..today"));
    assert_eq!(view.fields["path"], one("work/"));
}

#[test]
fn the_filter_field_asks_what_a_filter_selects_and_in_asks_what_it_lists() {
    let object = Object::Task(task(1));
    let oid = object.oid().clone();
    let mut fake = Fake::at("2026-11-20T12:00[UTC]");
    fake.selected.insert("today".into(), [oid.clone()].into());
    let by_field = Filter::read(&table(&[("filter", one("today"))])).unwrap();
    assert!(by_field.matches(&object, &fake));
    let mut child = task(2);
    child.base.parent = Some(oid);
    let by_parent = Filter::read(&table(&[("parent", one("in:today"))])).unwrap();
    assert!(!by_parent.matches(&Object::Task(child), &fake));
}

#[test]
fn a_filter_names_what_each_use_needs() {
    let t = table(&[
        ("filter", one("a")),
        ("parent", one("in:b")),
        ("due", one("on:c")),
    ]);
    let filter = Filter::read(&t).unwrap();
    assert_eq!(filter.selected_filters(), ["a"]);
    assert_eq!(filter.listed_filters(), ["b"]);
    assert_eq!(filter.event_filters(), ["c"]);
}
