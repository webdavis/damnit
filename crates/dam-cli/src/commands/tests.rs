use dam_application::testing::{config, oid, task_in, todoist};
use dam_domain::Object;

use super::*;
use crate::args::RmArgs;
use crate::testing::context;

fn removing_one_of_two_errands_over(ctx: &Context) -> String {
    for task in [
        task_in(1, "errands", "Home"),
        task_in(2, "errands", "Home"),
        task_in(9, "bins", "Home/errands"),
    ] {
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    let rm = Command::Rm(RmArgs {
        oid: Some(oid(1).as_str().into()),
        id: None,
    });
    dispatch_a_verb_that_opens_the_store(ctx, rm)
        .unwrap_err()
        .to_string()
}

#[test]
fn a_refused_command_offers_a_pull_when_a_remote_holds_the_unsettled_object() {
    let mut ctx = context();
    ctx.config = config();
    ctx.store.map_remote_id(&todoist(), &oid(9), "r9").unwrap();
    assert!(removing_one_of_two_errands_over(&ctx).contains("run: dam pull, or name it"));
}

#[test]
fn a_refused_command_offers_no_pull_when_no_remote_holds_the_unsettled_object() {
    let mut ctx = context();
    ctx.config = config();
    assert!(!removing_one_of_two_errands_over(&ctx).contains("dam pull"));
}
