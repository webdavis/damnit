use dam_domain::Object;
use dam_domain::Permission::{EventRead, TaskRead};

use crate::service::Dam;
use crate::testing::{Scenario, allowed, event, holder, task_in};

fn work_home_and_an_event() -> Scenario {
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "report", "work")));
    s.working(Object::Task(task_in(2, "laundry", "home")));
    s.working(Object::Task(task_in(3, "lathe", "workshop")));
    s.working(event(4));
    s
}

fn subjects(objects: &[Object]) -> Vec<&str> {
    objects.iter().map(|o| o.base().subject.as_str()).collect()
}

#[test]
fn a_token_lists_only_objects_inside_one_of_its_grants() {
    let s = work_home_and_an_event();
    let token = holder(vec![allowed(&[TaskRead], Some(&["work"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(
        subjects(&dam.list_within(&dam_domain::Path::root()).unwrap()),
        ["report"]
    );
}

#[test]
fn task_read_shows_no_event_and_event_read_shows_no_task() {
    let s = work_home_and_an_event();
    let tasks = holder(vec![allowed(&[TaskRead], None)]);
    let events = holder(vec![allowed(&[EventRead], None)]);
    let listed = |token| {
        let dam = Dam {
            access: token,
            ..s.dam()
        };
        dam.list_within(&dam_domain::Path::root()).unwrap().len()
    };
    assert_eq!(listed(&tasks), 3);
    assert_eq!(listed(&events), 1);
}

fn post_held_by_errands() -> Scenario {
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "errands", "home")));
    s.working(Object::Task(task_in(2, "post", "home/errands")));
    s
}

fn parent_seen_by(paths: &[&str]) -> Option<dam_domain::Oid> {
    let s = post_held_by_errands();
    let token = holder(vec![allowed(&[TaskRead], Some(paths))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let post = dam.seen_object(&crate::testing::oid(2)).unwrap().unwrap();
    post.base().parent.clone()
}

#[test]
fn a_token_is_shown_the_parent_it_may_see() {
    assert_eq!(parent_seen_by(&["home"]), Some(crate::testing::oid(1)));
}

#[test]
fn a_token_is_never_shown_a_parent_it_may_not_see() {
    assert_eq!(parent_seen_by(&["home/errands"]), None);
}
