use dam_domain::Permission::TaskRead;
use dam_domain::{Object, Order, Path, Sort};

use super::tests::{filter, subjects};
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, task_in};

fn order(sort: &[&str], offset: usize, limit: Option<usize>) -> Order {
    let texts: Vec<String> = sort.iter().map(|s| s.to_string()).collect();
    Order {
        sort: Sort::read(&texts).unwrap(),
        offset,
        limit,
    }
}

fn home_and_work() -> Scenario {
    let s = Scenario::default();
    let work = task_in(1, "report", "work");
    let mut chores = task_in(2, "chores", "home");
    chores.base.depends.push(work.base.oid.clone());
    s.working(Object::Task(work));
    s.working(Object::Task(chores));
    s.working(Object::Task(task_in(3, "laundry", "home")));
    s.working(Object::Task(task_in(4, "audit", "work")));
    s.working(Object::Task(task_in(5, "dishes", "home")));
    s
}

fn listed(dam: &Dam, order: &Order) -> Vec<String> {
    let out = dam
        .list_selected(&filter(&[]), order, &Path::root(), &[])
        .unwrap();
    subjects(&out).into_iter().map(String::from).collect()
}

#[test]
fn a_sort_reads_worked_out_fields_from_the_store_facts() {
    let s = home_and_work();
    let blocked_first = listed(&s.dam(), &order(&["blocked:desc", "subject"], 0, None));
    assert_eq!(
        blocked_first,
        ["chores", "audit", "dishes", "laundry", "report"]
    );
}

#[test]
fn offset_and_limit_count_only_what_a_token_may_read() {
    let s = home_and_work();
    let token = holder(vec![allowed(&[TaskRead], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let page = |offset, limit| listed(&dam, &order(&["subject"], offset, limit));
    assert_eq!(page(0, Some(2)), ["chores", "dishes"]);
    assert_eq!(page(2, Some(2)), ["laundry"]);
    assert_eq!(page(3, None), Vec::<String>::new());
}
