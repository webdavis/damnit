use dam_domain::Object;
use dam_domain::Permission::TaskRead;

use crate::config::Config;
use crate::service::Dam;
use crate::testing::{FakeFilterFunctions, Scenario, allowed, holder, saved_function, task_in};

fn laundry_waits_on_report() -> Scenario {
    let mut s = Scenario::default().configured(Config {
        filters: vec![saved_function("slipping", &[])],
        ..Config::default()
    });
    s.fixture.filter_functions = FakeFilterFunctions::keeping(&["laundry"]);
    let report = task_in(1, "report", "work");
    let mut laundry = task_in(2, "laundry", "home");
    laundry.base.depends.push(report.base.oid.clone());
    s.committed(Object::Task(report));
    s.committed(Object::Task(laundry));
    s
}

fn laundry_blocked(s: &Scenario) -> Option<bool> {
    let handed = s.fixture.filter_functions.handed.borrow();
    let (_, worked_out) = handed
        .iter()
        .find(|(subject, _)| subject == "laundry")
        .unwrap();
    worked_out.blocked
}

#[test]
fn match_is_handed_blocked_as_the_rows_read_it() {
    let s = laundry_waits_on_report();
    s.dam().list_saved("slipping").unwrap();
    assert_eq!(laundry_blocked(&s), Some(true));
}

#[test]
fn a_dependency_a_token_cannot_see_does_not_block_what_match_is_handed() {
    let s = laundry_waits_on_report();
    let token = holder(vec![allowed(&[TaskRead], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.list_saved("slipping").unwrap();
    assert_eq!(laundry_blocked(&s), Some(false));
}
