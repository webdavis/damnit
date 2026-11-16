use super::*;
use crate::{Oid, Task};

fn task(byte: u8, path: &str, labels: &[&str]) -> Object {
    let mut task = Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(byte)), "a task");
    task.base.path = Path::parse(path).unwrap();
    task.base.labels = labels.iter().map(|l| l.to_string()).collect();
    Object::Task(task)
}

fn path(text: &str) -> Path {
    Path::parse(text).unwrap()
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

#[test]
fn each_path_counts_the_objects_at_or_under_it_with_every_parent_listed() {
    let objects = [
        task(1, "work/cli", &[]),
        task(2, "work", &[]),
        task(3, "home/garden", &[]),
    ];
    let counted: Vec<(String, usize)> = paths_in_use(&objects, &Path::root())
        .into_iter()
        .map(|(p, n)| (p.to_string(), n))
        .collect();
    assert_eq!(
        counted,
        [
            ("home/".into(), 1),
            ("home/garden/".into(), 1),
            ("work/".into(), 2),
            ("work/cli/".into(), 1),
        ]
    );
}

#[test]
fn the_root_is_never_listed_as_a_path() {
    assert!(paths_in_use(&[task(1, "", &[])], &Path::root()).is_empty());
}

#[test]
fn inside_a_place_no_path_above_it_is_listed() {
    let objects = [task(1, "damnit/cli", &[]), task(2, "damnit", &[])];
    let listed: Vec<(Path, usize)> = paths_in_use(&objects, &path("damnit/"))
        .into_iter()
        .collect();
    assert_eq!(listed, [(path("damnit/"), 2), (path("damnit/cli/"), 1)]);
}
