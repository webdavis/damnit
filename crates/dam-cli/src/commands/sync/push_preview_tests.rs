use dam_application::{RemoteConfig, RemoteName};
use dam_domain::{Field, Kind, Object, Oid, Task};

use super::run_push;
use crate::args::PushArgs;
use crate::context::Context;
use crate::testing::context;

fn with_a_commit() -> Context {
    let mut ctx = context();
    ctx.config.remotes.push(RemoteConfig {
        name: RemoteName("t".into()),
        helper: "t".into(),
        url: "t::".into(),
        credentials: vec![],
        deadline: None,
        sync: Default::default(),
        path: None,
        except: Vec::new(),
    });
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(1));
    ctx.store
        .put(&Object::Task(Task::new(oid, "milk")))
        .unwrap();
    ctx.dam().add_all().unwrap();
    ctx.dam().commit("m").unwrap();
    ctx
}

fn previewed(ctx: &Context) -> crate::output::Report {
    run_push(
        ctx,
        PushArgs {
            remote: None,
            preview: true,
        },
    )
    .unwrap()
}

#[test]
fn a_remote_never_contacted_is_not_known_and_nothing_is_written() {
    let ctx = with_a_commit();
    let report = previewed(&ctx);
    assert!(
        report.human.contains("not known until dam pull t has run"),
        "{}",
        report.human
    );
    assert_eq!(report.data["preview"], true);
    assert_eq!(report.data["remotes"][0]["known"], false);
    assert_eq!(
        ctx.store.unpushed(&RemoteName("t".into())).unwrap().len(),
        1
    );
}

#[test]
fn a_contacted_remote_shows_what_would_be_sent_and_nothing_is_written() {
    let ctx = with_a_commit();
    let t = RemoteName("t".into());
    ctx.store.set_accepted_kinds(&t, &[Kind::Task]).unwrap();
    ctx.store
        .set_accepted_fields(&t, &[Field::Subject])
        .unwrap();
    let report = previewed(&ctx);
    let sent = &report.data["remotes"][0]["mutations"][0];
    assert_eq!(sent["op"], "create");
    assert_eq!(sent["subject"], "milk");
    assert!(
        report
            .human
            .trim_end()
            .ends_with("preview: 1 to send, 0 held, 0 skipped, nothing sent"),
        "{}",
        report.human
    );
    assert_eq!(ctx.store.unpushed(&t).unwrap().len(), 1);
    assert_eq!(
        ctx.store
            .remote_id(&t, &Oid::generate(&mut |x: &mut [u8]| x.fill(1)))
            .unwrap(),
        None
    );
}

#[test]
fn both_answers_match_the_published_schema() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/schemas/push-preview.schema.json"
    )))
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let ctx = with_a_commit();
    assert!(validator.is_valid(&previewed(&ctx).data));
    let t = RemoteName("t".into());
    ctx.store.set_accepted_kinds(&t, &[Kind::Task]).unwrap();
    ctx.store
        .set_accepted_fields(&t, &[Field::Subject])
        .unwrap();
    let data = previewed(&ctx).data;
    assert!(validator.is_valid(&data), "{data}");
}
