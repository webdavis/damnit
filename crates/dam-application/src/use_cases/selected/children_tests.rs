use dam_domain::Permission::TaskRead;
use dam_domain::{Completion, Object, Order, Path};

use super::tests::filter;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, oid, task_in};

fn trip_with_two_open_children_and_a_done_one() -> Scenario {
    let s = Scenario::default();
    s.working(Object::Task(task_in(1, "trip", "home")));
    for (b, subject, path) in [(2, "pack", "home"), (3, "book", "work")] {
        let mut child = task_in(b, subject, path);
        child.base.parent = Some(oid(1));
        s.working(Object::Task(child));
    }
    let mut done = task_in(4, "plan", "home");
    done.base.parent = Some(oid(1));
    done.completion = Completion::Done { at: None };
    s.working(Object::Task(done));
    s
}

fn trips_children_open(dam: &Dam) -> Option<usize> {
    let (listed, _) = dam
        .list_selected_noting(
            &filter(&[("subject", "trip")]),
            &Order::default(),
            &Path::root(),
            &[],
        )
        .unwrap();
    assert_eq!(listed.objects.len(), 1);
    listed.worked_out[0].children_open
}

#[test]
fn a_row_counts_its_open_children_even_when_the_filter_hides_them() {
    let s = trip_with_two_open_children_and_a_done_one();
    assert_eq!(trips_children_open(&s.dam()), Some(2));
}

#[test]
fn a_token_counts_only_the_children_it_may_read() {
    let s = trip_with_two_open_children_and_a_done_one();
    let token = holder(vec![allowed(&[TaskRead], Some(&["home"]))]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(trips_children_open(&dam), Some(1));
}
