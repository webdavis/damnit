mod support;

use support::hidden::{Hidden, text};
use support::tokens::{answer, refused};

fn without_the_item(stdout: &[u8]) -> serde_json::Value {
    let mut sent: serde_json::Value = serde_json::from_slice(stdout).unwrap();
    let fields = sent.as_object_mut().unwrap();
    fields.remove("hash");
    fields.remove("created");
    sent
}

fn plan_through_a_hidden_diary_to_the_report(h: &Hidden) -> (String, String) {
    let plan = h.task("plan", "work");
    let diary = h.task("diary", "home");
    h.owner(&["edit", &plan, "--depends", &diary]);
    h.owner(&["edit", &diary, "--depends", &h.report]);
    (plan, diary)
}

#[test]
fn a_tokens_dependency_that_closes_a_cycle_through_a_hidden_task_is_sent_like_any_other() {
    let _guard = support::guard(
        "a_tokens_dependency_that_closes_a_cycle_through_a_hidden_task_is_sent_like_any_other",
    );
    let h = Hidden::new();
    let (plan, diary) = plan_through_a_hidden_diary_to_the_report(&h);
    let line = ["edit", &h.report, "--depends", &plan, "--json"];
    let while_hidden = h.holding(&line);
    let sent = answer(&while_hidden, "sent.schema.json");
    assert_eq!(sent["answer"], "changed");
    h.sb.output_holding(&h.held, &["rm", sent["hash"].as_str().unwrap()]);
    h.owner(&["rm", &diary]);
    let once_missing = h.holding(&line);
    assert_eq!(
        (while_hidden.status.code(), text(&while_hidden.stderr)),
        (once_missing.status.code(), text(&once_missing.stderr))
    );
    assert_eq!(
        without_the_item(&while_hidden.stdout),
        without_the_item(&once_missing.stdout)
    );
}

#[test]
fn the_owner_cannot_add_a_tokens_dependency_that_closes_a_cycle() {
    let _guard = support::guard("the_owner_cannot_add_a_tokens_dependency_that_closes_a_cycle");
    let h = Hidden::new();
    let (plan, diary) = plan_through_a_hidden_diary_to_the_report(&h);
    h.owner(&["add", "-A"]);
    h.owner(&["commit", "-m", "seed"]);
    let line = ["edit", &h.report, "--depends", &plan, "--json"];
    let sent = answer(&h.holding(&line), "sent.schema.json");
    let hash = sent["hash"].as_str().unwrap();

    let out = h.sb.output(&["add", hash, "--json"]);
    assert_eq!(refused(&out), "item_cycle");
    let said = text(&h.sb.output(&["add", hash]).stderr);
    assert!(said.contains(&diary[..7]), "{said}");
    assert!(said.contains(&format!("dam rm {}", &hash[..7])), "{said}");

    h.owner(&["add", "-A"]);
    let (ok, status, err) = h.sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["waiting"][0]["hash"], hash);
}
