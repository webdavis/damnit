use crate::support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

const DENTIST: [&str; 3] = ["new", "Call the dentist", "--json"];

fn dentist_from_tool() -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:create"])]);
    let sent = answer(&sb.output_holding(&held, &DENTIST), "sent.schema.json");
    let hash = sent["hash"].as_str().unwrap().to_string();
    (sb, held, hash)
}

#[test]
fn a_token_withdraws_its_own_item_and_cannot_remove_another_senders() {
    let _guard = support::guard("a_token_withdraws_its_own_item_and_cannot_remove_another_senders");
    let (sb, held, hash) = dentist_from_tool();
    let claude = grant_to(
        &sb,
        "claude",
        2,
        vec![grant(&["pending:read", "task:read"])],
    );
    assert_eq!(
        refused(&sb.output_holding(&claude, &["rm", &hash, "--json"])),
        "not_your_item"
    );
    let out = sb.output_holding(&held, &["rm", &hash[..7]]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("withdrawn  {}", &hash[..7])
    );
    let again = answer(&sb.output_holding(&held, &DENTIST), "sent.schema.json");
    assert_eq!(again["answer"], "new");
}
