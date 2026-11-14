mod support;

use serde_json::{Value, json};
use support::hidden::text;
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to, refused};

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

fn nothing_moved(sb: &Sandbox, kind: &str) {
    let status = support::status_json(sb, &["--json"]);
    assert_eq!(status["staged"], json!([]), "{kind}");
    assert_eq!(status["unstaged"], json!([]), "{kind}");
    assert_eq!(status["waiting"], json!([]), "{kind}");
}

#[test]
fn reset_of_an_added_and_committed_item_is_refused_and_changes_nothing() {
    let _guard =
        support::guard("reset_of_an_added_and_committed_item_is_refused_and_changes_nothing");
    for kind in ["new", "removal"] {
        let (sb, hash, dentist) = added_and_committed(kind);

        let out = sb.output(&["reset", &hash]);
        assert_eq!(out.status.code(), Some(4), "{kind}: {out:?}");
        let said = text(&out.stderr);
        assert!(
            said.contains(&format!(
                "{} was added and committed; dam reset undoes only what is staged",
                &hash[..7]
            )),
            "{kind}: {said}"
        );
        assert_eq!(
            refused(&sb.output(&["reset", &hash, "--json"])),
            "item_already_committed",
            "{kind}"
        );

        nothing_moved(&sb, kind);
        let (shown, _, _) = sb.dam(&["show", &dentist]);
        assert_eq!(shown, kind == "new", "{kind}");
    }
}

#[test]
fn reset_of_an_added_new_item_still_staged_unstages_it() {
    let _guard = support::guard("reset_of_an_added_new_item_still_staged_unstages_it");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS)]);
    let hash = sent(&sb, &held, &["new", "Call the dentist"])["oid"]
        .as_str()
        .unwrap()
        .to_string();
    owner(&sb, &["add", &hash]);

    owner(&sb, &["reset", &hash]);

    let status = support::status_json(&sb, &["--json"]);
    assert_eq!(status["staged"], json!([]));
    assert_eq!(status["waiting"][0]["hash"], json!(hash));
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
