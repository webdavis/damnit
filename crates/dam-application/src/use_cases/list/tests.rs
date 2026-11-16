use crate::config::{Config, FilterConfig};
use crate::testing::{Scenario, oid};
use crate::testing::{fields, saved};
use dam_domain::{Categories, Category, Object, Path, Priority, Task, When};
use jiff::civil::date;

fn tasks_a_b_and_c_later(config: Config) -> Scenario {
    let s = Scenario::default().configured(config);
    let mut a = Task::new(oid(1), "a");
    a.due = Some(When::Day(date(2026, 9, 18)));
    a.base.labels.insert("deep".into());
    let mut b = Task::new(oid(2), "b");
    b.due = Some(When::Day(date(2026, 9, 10)));
    b.priority = Priority::HIGHEST;
    let mut c = Task::new(oid(3), "c");
    c.base.path = Path::parse("later").unwrap();
    for t in [a, b, c] {
        s.working(Object::Task(t));
    }
    s
}

fn effort_categories_and_a_today_filter() -> Config {
    Config {
        categories: Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["deep".into()],
            exclusive: true,
        }])
        .unwrap(),
        filters: vec![FilterConfig {
            name: "today".into(),
            table: dam_domain::FilterTable {
                any: vec![fields(&[("due", "today")]), fields(&[("overdue", "true")])],
                ..Default::default()
            },
            runs_a_function: false,
        }],
        ..Config::default()
    }
}

#[test]
fn the_root_lists_everything_by_path_then_earlier_due_first() {
    let s = tasks_a_b_and_c_later(effort_categories_and_a_today_filter());
    let out = s.dam().list_within(&dam_domain::Path::root()).unwrap();
    let subjects: Vec<&str> = out.iter().map(|o| o.base().subject.as_str()).collect();
    assert_eq!(subjects, vec!["b", "a", "c"]);
}

#[test]
fn a_saved_filter_runs_by_name() {
    let s = tasks_a_b_and_c_later(effort_categories_and_a_today_filter());
    let out = s.dam().list_saved("today").unwrap();
    assert_eq!(out.len(), 2);
}

#[test]
fn a_saved_filter_reads_a_quoted_value() {
    let config = Config {
        filters: vec![saved("kitchen", &[("path", "kitchen remodel/")])],
        ..Config::default()
    };
    let s = Scenario::default().configured(config);
    let mut t = Task::new(oid(1), "get quotes");
    t.base.path = Path::parse("kitchen remodel").unwrap();
    s.working(Object::Task(t));
    assert_eq!(s.dam().list_saved("kitchen").unwrap().len(), 1);
}
