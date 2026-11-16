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

fn a_waiting_set() -> (Context, String, String) {
    let (mut ctx, _) = three_tasks();
    dam(&ctx, &["add", "-A"]).unwrap();
    dam(&ctx, &["commit", "-m", "start"]).unwrap();
    ctx.access = holding(&[Permission::TaskRead, Permission::TaskChange]);
    let out = dam(
        &ctx,
        &["edit", "--query", "kind:task", "--subject", "x", "--as-one"],
    )
    .unwrap();
    let set = out.data["set"].as_str().unwrap().to_string();
    let member = out.data["changes"][0]["item"].as_str().unwrap().to_string();
    ctx.access = Box::new(dam_application::Owner);
    (ctx, set, member)
}

#[test]
fn the_owner_adds_a_set_whole_by_its_hash() {
    let (ctx, set, _) = a_waiting_set();
    dam(&ctx, &["add", &set]).unwrap();
    let status = dam(&ctx, &["status"]).unwrap();
    assert!(!status.human.contains(&set[..7]));
    let all = ctx.store.all().unwrap();
    assert!(all.iter().all(|o| o.base().subject == "x"));
}

#[test]
fn the_owner_declines_a_set_whole_and_a_member_alone_is_refused() {
    let (ctx, set, member) = a_waiting_set();
    let refused = dam(&ctx, &["add", &member]).unwrap_err();
    assert_eq!(refused.rule(), Some("item_in_a_set"));
    assert!(refused.to_string().contains(&set[..7]));
    let out = dam(&ctx, &["rm", &set]).unwrap();
    assert_eq!(out.data["set"], set.as_str());
    assert_eq!(out.data["items"].as_array().unwrap().len(), 3);
    assert!(
        out.data["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["outcome"] == "declined")
    );
}
