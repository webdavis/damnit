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
    assert_eq!(subjects(&dam.list(None).unwrap()), ["report"]);
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
        dam.list(None).unwrap().len()
    };
    assert_eq!(listed(&tasks), 3);
    assert_eq!(listed(&events), 1);
}

#[test]
fn a_query_runs_over_the_visible_objects_alone() {
    let s = work_home_and_an_event();
    let token = holder(vec![allowed(&[TaskRead], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert!(dam.list(Some("path:work/")).unwrap().is_empty());
}
