use super::*;
use crate::args::MvArgs;
use crate::testing::context;
use dam_domain::{Object, Oid, Path, Task};

fn filed(ctx: &Context, byte: u8, subject: &str) -> Oid {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(byte));
    ctx.store
        .put(&Object::Task(Task::new(oid.clone(), subject)))
        .unwrap();
    oid
}

#[test]
fn mv_relocates() {
    let ctx = context();
    let oid = filed(&ctx, 1, "x");
    run(
        &ctx,
        MvArgs {
            oid: oid.as_str().into(),
            path: Some(Path::parse("work/").unwrap()),
            parent: None,
        },
    )
    .unwrap();
    assert_eq!(
        ctx.store.get(&oid).unwrap().unwrap().base().path.as_str(),
        "work/"
    );
}

#[test]
fn mv_files_it_under_the_parent_a_prefix_names() {
    let ctx = context();
    let oid = filed(&ctx, 1, "x");
    let parent = filed(&ctx, 2, "errands");
    run(
        &ctx,
        MvArgs {
            oid: oid.as_str().into(),
            path: None,
            parent: Some("0202".into()),
        },
    )
    .unwrap();
    let moved = ctx.store.get(&oid).unwrap().unwrap();
    assert_eq!(moved.base().path.as_str(), "errands/");
    assert_eq!(moved.base().parent, Some(parent));
}

#[test]
fn mv_with_neither_a_path_nor_a_parent_names_both() {
    let ctx = context();
    let oid = filed(&ctx, 1, "x");
    let err = run(
        &ctx,
        MvArgs {
            oid: oid.as_str().into(),
            path: None,
            parent: None,
        },
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "dam mv needs a path or --parent: dam mv <oid> work/clients/, or dam mv <oid> --parent <oid>"
    );
}
