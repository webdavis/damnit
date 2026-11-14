mod support;

use support::hidden::{Hidden, text};
use support::tokens::{grant, grant_to};

fn without_ages(stdout: &[u8]) -> serde_json::Value {
    let mut status: serde_json::Value = serde_json::from_slice(stdout).unwrap();
    for row in status["waiting"].as_array_mut().unwrap() {
        row.as_object_mut().unwrap().remove("age");
    }
    status
}

fn full_status_alike_once_removed(h: &Hidden, held: &str, hidden: &str) {
    let line = ["status", "--full", "--json"];
    let while_hidden = h.sb.output_holding(held, &line);
    assert!(while_hidden.status.success(), "{while_hidden:?}");
    assert!(!text(&while_hidden.stdout).contains(hidden));
    h.owner(&["rm", hidden]);
    let once_missing = h.sb.output_holding(held, &line);
    assert_eq!(
        (while_hidden.status.code(), text(&while_hidden.stderr)),
        (once_missing.status.code(), text(&once_missing.stderr))
    );
    assert_eq!(
        without_ages(&while_hidden.stdout),
        without_ages(&once_missing.stdout)
    );
}

#[test]
fn a_tokens_full_status_hides_a_hidden_dependency_of_its_own_item() {
    let _guard = support::guard("a_tokens_full_status_hides_a_hidden_dependency_of_its_own_item");
    let h = Hidden::new();
    let diary = h.task("diary", "home");
    h.owner(&["edit", &h.report, "--depends", &diary]);
    let sent = h.holding(&["edit", &h.report, "--subject", "the report"]);
    assert!(sent.status.success(), "{sent:?}");
    full_status_alike_once_removed(&h, &h.held, &diary);
}

#[test]
fn a_tokens_full_status_hides_a_hidden_event_its_item_is_attached_to() {
    let _guard =
        support::guard("a_tokens_full_status_hides_a_hidden_event_its_item_is_attached_to");
    let h = Hidden::new();
    let dentist = h.event("dentist", "home");
    h.owner(&["edit", &h.report, "--attach", &dentist]);
    let sent = h.holding(&["edit", &h.report, "--subject", "the report"]);
    assert!(sent.status.success(), "{sent:?}");
    full_status_alike_once_removed(&h, &h.held, &dentist);
}

#[test]
fn another_senders_item_under_pending_read_hides_a_hidden_dependency() {
    let _guard =
        support::guard("another_senders_item_under_pending_read_hides_a_hidden_dependency");
    let h = Hidden::new();
    let diary = h.task("diary", "home");
    h.owner(&["edit", &h.report, "--depends", &diary]);
    let reader = grant_to(
        &h.sb,
        "reader",
        2,
        vec![grant(&["pending:read"]).paths(&["work"])],
    );
    let sent = h.holding(&["edit", &h.report, "--subject", "the report"]);
    assert!(sent.status.success(), "{sent:?}");
    full_status_alike_once_removed(&h, &reader, &diary);
}
