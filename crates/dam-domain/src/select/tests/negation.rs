use super::fake::Fake;
use super::objects::{due, event, labelled, task};
use crate::select::Condition;
use crate::{Object, Path};

fn holds(key: &str, value: &str, object: &Object) -> bool {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    Condition::read(key, value).unwrap().holds(object, &fake)
}

#[test]
fn not_means_none_of_and_passes_an_empty_field() {
    let both = labelled(Object::Task(task(1)), &["bug", "ui"]);
    let bare = Object::Task(task(2));
    assert!(!holds("labels", "not:bug", &both));
    assert!(!holds("labels", "not:waiting,ui", &both));
    assert!(holds("labels", "not:waiting,someday", &both));
    assert!(holds("labels", "not:bug", &bare));
    assert!(holds("due", "not:2026-11", &bare));
}

#[test]
fn not_none_is_the_one_negation_that_refuses_an_empty_field() {
    let dated = Object::Task(due(task(1), "2026-11-20".parse().unwrap()));
    let bare = Object::Task(task(2));
    assert!(holds("due", "not:none", &dated));
    assert!(!holds("due", "not:none", &bare));
    assert!(holds("due", "none", &bare));
}

#[test]
fn none_in_a_list_is_one_more_choice() {
    let bug = labelled(Object::Task(task(1)), &["bug"]);
    let ui = labelled(Object::Task(task(2)), &["ui"]);
    let bare = Object::Task(task(3));
    assert!(holds("labels", "none,bug", &bug));
    assert!(holds("labels", "none,bug", &bare));
    assert!(!holds("labels", "none,bug", &ui));
}

#[test]
fn a_field_the_kind_lacks_fails_a_value_and_passes_a_not() {
    let e = Object::Event(event(1, "2026-11-20T09:00[UTC]"));
    assert!(!holds("due", "none", &e));
    assert!(!holds("due", "not:none", &e));
    assert!(holds("due", "not:2026-11", &e));
    assert!(!holds("priority", "4", &e));
}

#[test]
fn a_done_condition_never_removes_an_event() {
    let e = Object::Event(event(1, "2026-11-20T09:00[UTC]"));
    assert!(holds("done", "true", &e) && holds("done", "false", &e));
}

#[test]
fn the_escape_takes_the_rest_exactly_as_written() {
    let literal = labelled(Object::Task(task(1)), &["none"]);
    let bare = Object::Task(task(2));
    assert!(holds("labels", "=none", &literal));
    assert!(!holds("labels", "=none", &bare));
    assert!(holds("labels", "not:=none", &bare));
    assert!(!holds("labels", "not:=none", &literal));
    let mut comma = task(3);
    comma.base.path = Path::parse("a,b/c").unwrap();
    assert!(holds("path", "=a,b/", &Object::Task(comma)));
}
