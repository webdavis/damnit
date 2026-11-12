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
fn a_waiting_change_in_status_text_names_its_hash_and_the_task_it_changes() {
    let _guard =
        support::guard("a_waiting_change_in_status_text_names_its_hash_and_the_task_it_changes");
    let (sb, rent, hash) = rent_and_a_retitle_from_vpt();
    let status = text(&sb.output(&["status"]).stdout);
    let row = format!(
        "  changed  {}  {}  -Pay rent  vpt  ",
        &hash[..7],
        &rent[..7]
    );
    assert!(status.lines().any(|l| l.starts_with(&row)), "{status}");
}
