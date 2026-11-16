use std::collections::BTreeSet;

use dam_application::testing::{allowed, holder};
use dam_application::{Dam, NewTask};
use dam_domain::{Path, Permission};

use crate::args::AddArgs;
use crate::context::Context;
use crate::testing::context;

fn sent_by_a_token(ctx: &Context) -> String {
    let token = holder(vec![allowed(
        &[Permission::TaskRead, Permission::TaskCreate],
        None,
    )]);
    let dam = Dam {
        access: &token,
        ..ctx.dam()
    };
    let landed = dam
        .new_task(NewTask {
            parent: None,
            subject: "milk".into(),
            path: Path::root(),
            priority: Default::default(),
            due: None,
            deadline: None,
            labels: BTreeSet::new(),
            body: String::new(),
            id: None,
        })
        .unwrap();
    match landed {
        dam_application::Landed::Sent(sent) => sent.item.hash.to_string(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_preview_shows_what_would_be_staged_and_taken_and_writes_nothing() {
    let ctx = context();
    let hash = sent_by_a_token(&ctx);
    let (objects, items) = (ctx.store.all().unwrap(), ctx.store.items().unwrap());
    let report = super::run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
            preview: true,
        },
    )
    .unwrap();
    assert_eq!(report.data["preview"], true);
    assert_eq!(report.data["taken"][0], hash);
    assert_eq!(report.data["staged"][0]["subject"], "milk");
    assert!(report.human.trim_end().ends_with("nothing written"));
    assert_eq!(ctx.store.all().unwrap(), objects);
    assert_eq!(ctx.store.items().unwrap(), items);
    assert!(ctx.store.staged().unwrap().is_empty());
}

#[test]
fn a_run_without_preview_says_nothing_of_a_preview() {
    let ctx = context();
    sent_by_a_token(&ctx);
    let report = super::run_add(
        &ctx,
        AddArgs {
            oids: vec![],
            all: true,
            preview: false,
        },
    )
    .unwrap();
    assert!(report.data.get("preview").is_none());
    assert!(report.data.get("taken").is_none());
}
