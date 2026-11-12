use dam_domain::{Change, Completion, Object, Oid, Priority, Task};

use super::ChangeRow;
use crate::view::Line;

fn text(lines: Vec<Line>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| match line {
            Line::Row(l) | Line::Whole(l) | Line::Removed(l) | Line::Added(l) => l,
        })
        .collect()
}

#[test]
fn each_changed_field_reads_as_its_name_and_value_on_both_lines() {
    let mut before = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "a");
    before.priority = Priority::new(1).unwrap();
    before.base.labels.insert("errand".into());
    before.completion = Completion::Done { at: None };
    let mut after = before.clone();
    after.priority = Priority::new(3).unwrap();
    after.base.labels.clear();
    after.completion = Completion::Open;
    let change = Change::between(Some(Object::Task(before)), Some(Object::Task(after))).unwrap();
    assert_eq!(
        text(ChangeRow::new(&change, None, false).own_lines("", "you")),
        [
            "changed  0101010  -a  labels: errand  priority: 1  done: true  you",
            "                  +a  labels: -  priority: 3  done: false",
        ]
    );
}
