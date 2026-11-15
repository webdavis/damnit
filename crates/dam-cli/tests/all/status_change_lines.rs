use crate::support;

use support::hidden::{full_oid, text};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

#[test]
fn a_waiting_change_keeps_its_hash_target_sender_and_age_on_the_minus_line() {
    let _guard =
        support::guard("a_waiting_change_keeps_its_hash_target_sender_and_age_on_the_minus_line");
    let sb = Sandbox::new();
    let rent = full_oid(&sb, &sb.new_object(&["Pay rent"]));
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "task:change"])]);
    let line = ["edit", &rent, "--subject", "Pay the rent", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let hash = &sent["hash"].as_str().unwrap()[..7];
    let status = text(&sb.output(&["status"]).stdout);
    let lines: Vec<&str> = status.lines().collect();
    let minus = lines.iter().position(|l| l.contains("-Pay rent")).unwrap();
    assert!(
        lines[minus].starts_with(&format!(
            "  changed  {hash}  {}  -Pay rent  tool  ",
            &rent[..7]
        )),
        "{status}"
    );
    assert_eq!(lines[minus + 1], format!("{}+Pay the rent", " ".repeat(29)));
}
