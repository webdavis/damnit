use crate::support;

use serde_json::{Value, json};
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

const RIGHTS: [&str; 3] = ["task:create", "task:read", "task:remove"];

fn owner(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn sent(sb: &Sandbox, held: &str, line: &[&str]) -> Value {
    let mut line = line.to_vec();
    line.push("--json");
    let out = sb.output_holding(held, &line);
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn added_and_committed(kind: &str) -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS)]);
    let dentist = sent(&sb, &held, &["new", "Call the dentist"])["oid"]
        .as_str()
        .unwrap()
        .to_string();
    let hash = if kind == "new" {
        dentist.clone()
    } else {
        owner(&sb, &["add", &dentist]);
        owner(&sb, &["commit", "-m", "dentist"]);
        sent(&sb, &held, &["rm", &dentist])["item"]
            .as_str()
            .unwrap()
            .to_string()
    };
    owner(&sb, &["add", &hash]);
    owner(&sb, &["commit", "-m", kind]);
    (sb, hash, dentist)
}

#[test]
fn reset_of_a_committed_new_items_object_still_unstages_the_owners_edit() {
    let _guard =
        support::guard("reset_of_a_committed_new_items_object_still_unstages_the_owners_edit");
    let (sb, _, dentist) = added_and_committed("new");
    owner(&sb, &["edit", &dentist, "--body", "before noon"]);
    owner(&sb, &["add", &dentist]);

    let (ok, out, err) = sb.dam(&["reset", &dentist]);
    assert!(ok, "{err}");
    assert_eq!(out.trim_end(), format!("unstaged {}", &dentist[..7]));

    let status = support::status_json(&sb, &["--json"]);
    assert_eq!(status["staged"], json!([]));
    assert_eq!(status["unstaged"][0]["fields"], json!(["body"]));
}
