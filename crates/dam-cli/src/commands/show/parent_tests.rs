use dam_domain::{Object, Oid, Path, Permission, Task};

use crate::args::ShowArgs;
use crate::output::Report;
use crate::testing::{context, holding};

fn task(byte: u8, subject: &str, path: &str, parent: Option<u8>) -> Object {
    let mut task = Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(byte)), subject);
    task.base.path = Path::parse(path).unwrap();
    task.base.parent = parent.map(|p| Oid::generate(&mut |b: &mut [u8]| b.fill(p)));
    Object::Task(task)
}

fn shown(byte: u8, paths: Option<&[&str]>) -> Report {
    let mut ctx = context();
    ctx.store.put(&task(1, "errands", "home", None)).unwrap();
    ctx.store
        .put(&task(2, "post", "home/errands", Some(1)))
        .unwrap();
    if let Some(paths) = paths {
        let mut token = holding(&[Permission::TaskRead]);
        token.grants.grants[0].paths =
            Some(paths.iter().map(|p| Path::parse(p).unwrap()).collect());
        ctx.access = token;
    }
    let id = format!("{byte:02x}{byte:02x}{byte:02x}{byte:02x}");
    super::run_show(&ctx, ShowArgs { id }).unwrap()
}

#[test]
fn show_prints_the_parents_short_id_and_subject() {
    assert!(shown(2, None).human.contains("parent: 0101010 errands"));
}

#[test]
fn show_prints_no_parent_row_for_an_object_without_one() {
    assert!(!shown(1, None).human.contains("parent:"));
}

#[test]
fn show_prints_the_parent_a_token_may_see() {
    assert!(
        shown(2, Some(&["home"]))
            .human
            .contains("parent: 0101010 errands")
    );
}

#[test]
fn show_never_names_a_parent_a_token_may_not_see() {
    let report = shown(2, Some(&["home/errands"]));
    assert!(!report.human.contains("parent:"));
    assert!(report.data.get("parent").is_none());
}
