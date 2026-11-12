mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn committed_task(sb: &Sandbox, subject: &str) -> String {
    let short = sb.new_object(&[subject]);
    for line in [vec!["add", "-A"], vec!["commit", "-m", "seed"]] {
        let (ok, _, err) = sb.dam(&line);
        assert!(ok, "{err}");
    }
    let (_, out, _) = sb.dam(&["show", &short, "--json"]);
    serde_json::from_str::<serde_json::Value>(&out).unwrap()["oid"]
        .as_str()
        .unwrap()
        .to_string()
}

fn owner_subject(sb: &Sandbox, oid: &str) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["show", oid, "--json", "--no-pull"]);
    assert!(ok, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).unwrap()
}

#[test]
fn a_tokens_edit_answers_changed_and_the_owners_task_is_untouched() {
    let _guard = support::guard("a_tokens_edit_answers_changed_and_the_owners_task_is_untouched");
    let sb = Sandbox::new();
    let oid = committed_task(&sb, "Pay rent");
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let line = ["edit", &oid[..7], "--subject", "Pay the rent", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    assert_eq!(sent["answer"], "changed");
    assert_eq!(sent["item"], "change");
    assert_eq!(sent["oid"], oid.as_str());
    assert_ne!(sent["hash"], oid.as_str());
    assert_eq!(sent["subject"], "Pay the rent");
    assert_eq!(owner_subject(&sb, &oid)["subject"], "Pay rent");
}

#[test]
fn a_tokens_done_answers_changed_and_force_stays_the_owners() {
    let _guard = support::guard("a_tokens_done_answers_changed_and_force_stays_the_owners");
    let sb = Sandbox::new();
    let oid = committed_task(&sb, "Pay rent");
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:complete\""));
    let out = sb.output_holding(&held, &["done", &oid]);
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with("changed  ") && text.trim_end().ends_with("Pay rent"),
        "{text}"
    );
    assert_eq!(owner_subject(&sb, &oid)["task"]["done"], false);
    let forced = sb.output_holding(&held, &["done", &oid, "--force", "--json"]);
    assert_eq!(refused(&forced), "not_open_to_tokens");
}
