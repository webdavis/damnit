mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

const DENTIST: [&str; 3] = ["new", "Call the dentist", "--json"];

fn dentist_from_vpt() -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:create\""));
    let sent = answer(&sb.output_holding(&held, &DENTIST), "sent.schema.json");
    let hash = sent["hash"].as_str().unwrap().to_string();
    (sb, held, hash)
}

#[test]
fn the_owner_declines_an_item_and_the_same_task_sent_again_is_refused() {
    let _guard =
        support::guard("the_owner_declines_an_item_and_the_same_task_sent_again_is_refused");
    let (sb, held, hash) = dentist_from_vpt();
    let declined = answer(
        &sb.output(&["rm", &hash[..7], "--json"]),
        "settled.schema.json",
    );
    assert_eq!(declined["outcome"], "declined");
    assert_eq!(declined["hash"], hash.as_str());
    assert_eq!(
        refused(&sb.output_holding(&held, &DENTIST)),
        "item_declined"
    );
    assert_eq!(
        refused(&sb.output(&["rm", &hash, "--json"])),
        "no_such_object"
    );
    let (_, status, _) = sb.dam(&["status", "--json"]);
    assert!(!status.contains(&hash), "{status}");
}

#[test]
fn a_token_withdraws_its_own_item_and_cannot_remove_another_senders() {
    let _guard = support::guard("a_token_withdraws_its_own_item_and_cannot_remove_another_senders");
    let (sb, held, hash) = dentist_from_vpt();
    let claude = grant_to(
        &sb,
        "claude",
        2,
        "[[token.claude.grant]]\nallow = [\"pending:read\"]\n",
    );
    assert_eq!(
        refused(&sb.output_holding(&claude, &["rm", &hash, "--json"])),
        "not_your_item"
    );
    let out = sb.output_holding(&held, &["rm", &hash[..7]]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("withdrawn  {}  Call the dentist  vpt", &hash[..7])
    );
    let again = answer(&sb.output_holding(&held, &DENTIST), "sent.schema.json");
    assert_eq!(again["answer"], "new");
}
