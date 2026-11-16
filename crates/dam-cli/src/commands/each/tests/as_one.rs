use dam_domain::Permission;

use super::*;
use crate::testing::holding;

pub(super) fn token_context() -> (Context, Vec<String>) {
    let (mut ctx, oids) = three_tasks();
    ctx.access = holding(&[Permission::TaskRead, Permission::TaskChange]);
    (ctx, oids)
}

#[test]
fn a_tokens_as_one_run_answers_the_one_set_its_items_wait_in() {
    let (ctx, _) = token_context();
    let out = dam(
        &ctx,
        &["edit", "--query", "kind:task", "--subject", "x", "--as-one"],
    )
    .unwrap();
    let set = out.data["set"].as_str().expect("the answer names the set");
    assert_eq!(out.data["changes"].as_array().unwrap().len(), 3);
    assert!(last_line(&out).contains(&set[..7]));
}

#[test]
fn as_one_from_the_owner_is_a_usage_problem() {
    let (ctx, oids) = three_tasks();
    let refused = dam(&ctx, &["rm", &oids[0], &oids[1], "--as-one"]).unwrap_err();
    assert_eq!(refused.exit_code(), 2);
    assert!(refused.to_string().contains("--as-one"));
}
