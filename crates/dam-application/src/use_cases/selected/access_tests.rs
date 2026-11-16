use dam_domain::Permission::TaskRead;
use dam_domain::{Object, Order, Path};

use super::tests::{filter, subjects};
use crate::config::Config;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, task_in};

fn home_waits_on_work() -> Scenario {
    let s = Scenario::default().configured(Config {
        filters: vec![crate::testing::saved("all", &[])],
        ..Config::default()
    });
    let work = task_in(1, "report", "work");
    let mut home = task_in(2, "laundry", "home");
    home.base.depends.push(work.base.oid.clone());
    s.working(Object::Task(work));
    s.working(Object::Task(home));
    s
}

fn listed_by_home_token(s: &Scenario, fields: &[(&str, &str)]) -> usize {
    let token = holder(vec![allowed(&[TaskRead], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    dam.list_selected(&filter(fields), &Order::default(), &Path::root(), &[])
        .unwrap()
        .len()
}

#[test]
fn a_token_lists_and_runs_saved_filters_over_what_it_may_read() {
    let s = home_waits_on_work();
    assert_eq!(listed_by_home_token(&s, &[]), 1);
    assert_eq!(listed_by_home_token(&s, &[("filter", "all")]), 1);
    assert_eq!(listed_by_home_token(&s, &[("path", "work")]), 0);
}

#[test]
fn a_task_hidden_from_a_token_never_shows_as_holding_one_up() {
    let s = home_waits_on_work();
    assert_eq!(listed_by_home_token(&s, &[("blocked", "true")]), 0);
    let owner = s
        .dam()
        .list_selected(
            &filter(&[("blocked", "true")]),
            &Order::default(),
            &Path::root(),
            &[],
        )
        .unwrap();
    assert_eq!(subjects(&owner), ["laundry"]);
}
