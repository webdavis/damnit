mod support;

use support::hidden::{full_oid, text};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn rent_and_a_retitle_from_vpt() -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let rent = full_oid(&sb, &sb.new_object(&["Pay rent"]));
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let line = ["edit", &rent, "--subject", "Pay the rent", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    (sb, rent, sent["hash"].as_str().unwrap().to_string())
}

#[test]
fn declining_a_change_names_the_task_it_changes_and_who_sent_it() {
    let _guard = support::guard("declining_a_change_names_the_task_it_changes_and_who_sent_it");
    let (sb, rent, hash) = rent_and_a_retitle_from_vpt();
    assert_eq!(
        text(&sb.output(&["rm", &hash[..7]]).stdout),
        format!(
            "declined  {}  {}  Pay the rent  vpt\n",
            &hash[..7],
            &rent[..7]
        )
    );
}

#[test]
fn declining_a_new_object_names_its_hash_once_and_who_sent_it() {
    let _guard = support::guard("declining_a_new_object_names_its_hash_once_and_who_sent_it");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:create\""));
    let sent = answer(
        &sb.output_holding(&held, &["new", "Call the dentist", "--json"]),
        "sent.schema.json",
    );
    let hash = sent["hash"].as_str().unwrap();
    assert_eq!(
        text(&sb.output(&["rm", &hash[..7]]).stdout),
        format!("declined  {}  Call the dentist  vpt\n", &hash[..7])
    );
}
