mod support;

use support::hidden::text;
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

#[test]
fn the_owner_cannot_add_a_tokens_completion_of_a_task_that_since_waits_on_an_open_one() {
    let _guard = support::guard(
        "the_owner_cannot_add_a_tokens_completion_of_a_task_that_since_waits_on_an_open_one",
    );
    let sb = Sandbox::new();
    let rent = sb.new_object(&["Pay rent"]);
    let keys = sb.new_object(&["Find the keys"]);
    let held = grant_to(&sb, "vpt", 1, &grant("\"task:read\", \"task:complete\""));
    let sent = answer(
        &sb.output_holding(&held, &["done", &rent, "--json"]),
        "sent.schema.json",
    );
    let hash = sent["hash"].as_str().unwrap();
    for line in [
        vec!["edit", &rent, "--depends", &keys],
        vec!["add", &rent, &keys],
        vec!["commit", "-m", "rent waits on the keys"],
    ] {
        let (ok, _, err) = sb.dam(&line);
        assert!(ok, "{err}");
    }

    assert_eq!(
        refused(&sb.output(&["add", hash, "--json"])),
        "item_blocked"
    );
    let said = text(&sb.output(&["add", hash]).stderr);
    assert!(said.contains(&keys[..7]), "{said}");
    assert!(said.contains(&format!("dam done {}", &rent[..7])), "{said}");
    assert!(said.contains(&format!("dam rm {}", &hash[..7])), "{said}");

    let (ok, _, err) = sb.dam(&["add", "-A"]);
    assert!(ok, "{err}");
    let (_, status, _) = sb.dam(&["status", "--json"]);
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["waiting"][0]["hash"], hash);
}
