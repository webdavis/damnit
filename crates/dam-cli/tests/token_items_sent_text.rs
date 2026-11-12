mod support;

use support::hidden::{full_oid, text};
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

#[test]
fn a_tokens_text_answer_to_an_edit_names_the_item_and_the_task_it_changes() {
    let _guard =
        support::guard("a_tokens_text_answer_to_an_edit_names_the_item_and_the_task_it_changes");
    let sb = Sandbox::new();
    let rent = full_oid(&sb, &sb.new_object(&["Pay rent"]));
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let edit = ["edit", &rent, "--subject", "Pay the rent"];
    let said = text(&sb.output_holding(&held, &edit).stdout);
    let json = sb.output_holding(&held, &[&edit[..], &["--json"]].concat());
    let sent: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let hash = sent["hash"].as_str().unwrap();
    assert_eq!(
        said,
        format!("changed  {}  {}  Pay the rent\n", &hash[..7], &rent[..7])
    );
}
