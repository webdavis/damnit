use std::collections::BTreeMap;

use dam_domain::{Categories, Category, Object, When};

use crate::config::Config;
use crate::testing::{Scenario, task_in};

#[test]
fn an_object_alone_gives_its_category_overdue_and_days_and_leaves_blocked_and_children_unread() {
    let s = Scenario::default().configured(Config {
        categories: Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["deep".into(), "light".into()],
            exclusive: true,
        }])
        .unwrap(),
        ..Config::default()
    });
    let mut late = task_in(1, "late", "work");
    late.due = Some(When::Day(jiff::civil::date(2000, 1, 1)));
    late.base.labels.insert("deep".into());
    late.base.depends.push(task_in(2, "first", "work").base.oid);
    let worked = s.dam().worked_out_from_object(&Object::Task(late));
    assert_eq!(
        worked.category,
        BTreeMap::from([("effort".to_string(), vec!["deep".to_string()])])
    );
    assert_eq!(worked.overdue, Some(true));
    assert_eq!(worked.blocked, None);
    assert_eq!(worked.children_open, None);
    assert!(worked.days_until_due.is_some_and(|days| days < 0));
}
