use dam_application::{Notice, RemoteName};
use dam_domain::Permission::RemoteRead;

use crate::args::StatusArgs;
use crate::testing::{context, holding};

fn notices_shown(ctx: &crate::context::Context) -> Vec<serde_json::Value> {
    let report = super::run_status(ctx, StatusArgs { full: false }).unwrap();
    report.data["notices"].as_array().unwrap().clone()
}

#[test]
fn a_task_pulled_failure_shows_to_the_owner_and_never_to_a_token() {
    let mut ctx = context();
    ctx.store
        .add_notice(&Notice::TaskPulledFailed {
            remote: RemoteName("todoist".into()),
            failed: 0,
            not_reached: 3,
            error: None,
        })
        .unwrap();
    assert_eq!(notices_shown(&ctx).len(), 1);
    ctx.access = holding(&[RemoteRead]);
    assert_eq!(notices_shown(&ctx), Vec::<serde_json::Value>::new());
}
