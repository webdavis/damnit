use crate::args::{AddArgs, CommitArgs, ShowArgs};
use crate::commands::commit::run_commit;
use crate::commands::stage::run_add;
use crate::error::CliError;
use crate::testing::context;
use dam_domain::{Object, Oid, Task};

#[test]
fn show_finds_an_object_or_a_commit_by_prefix() {
    let ctx = context();
    let oid = Oid::generate(&mut |x: &mut [u8]| {
        x.fill(A_BYTE_APART_FROM_THE_COMMIT_ID_COUNTING_RANDOM_FILLS_FIRST)
    });
    ctx.store
        .put(&Object::Task(Task::new(oid.clone(), "milk")))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &ctx,
        CommitArgs {
            message: "first".into(),
        },
    )
    .unwrap();
    let object = super::run_show(
        &ctx,
        ShowArgs {
            id: oid.short().into(),
        },
    )
    .unwrap();
    assert_eq!(object.data["subject"], "milk");
    assert!(object.human.contains("milk"));
    let id = ctx.store.log().unwrap()[0].id.to_string();
    let commit = super::run_show(&ctx, ShowArgs { id: id[..8].into() }).unwrap();
    assert_eq!(commit.data["message"], "first");
    assert!(matches!(
        super::run_show(&ctx, ShowArgs { id: "ffff".into() }),
        Err(CliError::UseCase(_))
    ));
}

const A_BYTE_APART_FROM_THE_COMMIT_ID_COUNTING_RANDOM_FILLS_FIRST: u8 = 3;

struct TwinPrefixRandom(std::cell::Cell<u8>);
impl dam_application::Randomness for TwinPrefixRandom {
    fn fill(&self, buf: &mut [u8]) {
        buf.fill(0xab);
        let len = buf.len();
        buf[len - 1] = self.0.get();
        self.0.set(self.0.get().wrapping_add(1));
    }
}

#[test]
fn an_ambiguous_commit_prefix_names_every_match() {
    let mut ctx = context();
    ctx.random = Box::new(TwinPrefixRandom(std::cell::Cell::new(1)));
    ctx.store
        .put(&Object::Task(Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(9)),
            "a",
        )))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &ctx,
        CommitArgs {
            message: "first".into(),
        },
    )
    .unwrap();
    ctx.store
        .put(&Object::Task(Task::new(
            Oid::generate(&mut |x: &mut [u8]| x.fill(10)),
            "b",
        )))
        .unwrap();
    run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
        },
    )
    .unwrap();
    run_commit(
        &ctx,
        CommitArgs {
            message: "second".into(),
        },
    )
    .unwrap();
    let err = super::run_show(
        &ctx,
        ShowArgs {
            id: "abababab".into(),
        },
    )
    .unwrap_err();
    let ids: Vec<String> = ctx
        .store
        .log()
        .unwrap()
        .iter()
        .map(|c| c.id.to_string())
        .collect();
    assert_eq!(err.exit_code(), 2);
    assert_eq!(err.document()["error"]["rule"], "ambiguous_prefix");
    assert_eq!(err.document()["error"]["oids"], serde_json::json!(ids));
}
