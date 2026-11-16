use super::*;
use crate::{Oid, Path, Task};

fn task(byte: u8, path: &str, labels: &[&str]) -> Object {
    let mut task = Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(byte)), "a task");
    task.base.path = Path::parse(path).unwrap();
    task.base.labels = labels.iter().map(|l| l.to_string()).collect();
    Object::Task(task)
}

#[test]
fn each_label_counts_the_objects_that_carry_it_sorted_by_name() {
    let objects = [
        task(1, "", &["urgent", "bug"]),
        task(2, "", &["urgent"]),
        task(3, "", &[]),
    ];
    let counted: Vec<(String, usize)> = labels_in_use(&objects).into_iter().collect();
    assert_eq!(counted, [("bug".into(), 1), ("urgent".into(), 2)]);
}
