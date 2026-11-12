mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn owner(sb: &Sandbox, args: &[&str], schema: &str) -> serde_json::Value {
    let mut line = args.to_vec();
    line.push("--json");
    answer(&sb.output(&line), schema)
}

fn seeded_with_rent() -> (Sandbox, String, String) {
    let sb = Sandbox::new();
    let rent = sb.new_object(&["Pay rent"]);
    owner(&sb, &["add", "-A"], "add.schema.json");
    owner(&sb, &["commit", "-m", "seed"], "commit.schema.json");
    let held = grant_to(
        &sb,
        "vpt",
        1,
        &grant("\"task:read\", \"task:create\", \"task:change\""),
    );
    (sb, held, rent)
}

fn send(sb: &Sandbox, held: &str, args: &[&str]) -> String {
    let mut line = args.to_vec();
    line.push("--json");
    let sent = answer(&sb.output_holding(held, &line), "sent.schema.json");
    sent["hash"].as_str().unwrap().to_string()
}

#[test]
fn the_owner_adds_and_commits_a_tokens_items_and_history_names_the_token() {
    let _guard =
        support::guard("the_owner_adds_and_commits_a_tokens_items_and_history_names_the_token");
    let (sb, held, rent) = seeded_with_rent();
    let dentist = send(&sb, &held, &["new", "Call the dentist", "--path", "inbox"]);
    let retitle = send(&sb, &held, &["edit", &rent, "--subject", "Pay the rent"]);

    let added = owner(
        &sb,
        &["add", &dentist[..6], &retitle[..6]],
        "add.schema.json",
    );
    assert_eq!(added["staged"][0]["oid"], dentist.as_str());
    assert_eq!(added["staged"][0]["from"], "vpt");
    assert_eq!(added["staged"][1]["subject"], "Pay the rent");
    let commit = owner(&sb, &["commit", "-m", "tool work"], "commit.schema.json");
    assert_eq!(commit["changes"][0]["from"], "vpt");
    assert_eq!(commit["changes"][0]["channel"], "local");

    let log = owner(&sb, &["log"], "log.schema.json");
    assert_eq!(log["commits"][0]["changes"][1]["from"], "vpt");
    assert_eq!(
        log["commits"][1]["changes"][0]["from"],
        serde_json::Value::Null
    );
    let (_, text, _) = sb.dam(&["log"]);
    assert!(
        text.lines()
            .next()
            .unwrap()
            .ends_with("2 changes  vpt  tool work"),
        "{text}"
    );
    let (_, shown, _) = sb.dam(&["show", &dentist]);
    assert!(shown.contains("Call the dentist"), "{shown}");
    let id = commit["id"].as_str().unwrap();
    let (_, shown, _) = sb.dam(&["show", &id[..7]]);
    assert!(shown.contains("Call the dentist  vpt"), "{shown}");
}

#[test]
fn a_change_to_a_field_the_owner_has_not_saved_stops_the_add_and_add_all_leaves_it_waiting() {
    let _guard = support::guard(
        "a_change_to_a_field_the_owner_has_not_saved_stops_the_add_and_add_all_leaves_it_waiting",
    );
    let (sb, held, rent) = seeded_with_rent();
    let retitle = send(&sb, &held, &["edit", &rent, "--subject", "Pay the rent"]);
    let dentist = send(&sb, &held, &["new", "Call the dentist"]);
    let (ok, _, err) = sb.dam(&["edit", &rent, "--subject", "Pay rent now"]);
    assert!(ok, "{err}");
    assert_eq!(
        refused(&sb.output(&["add", &retitle, "--json"])),
        "item_clash"
    );
    let added = owner(&sb, &["add", "-A"], "add.schema.json");
    let staged: Vec<&str> = added["staged"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["subject"].as_str().unwrap())
        .collect();
    assert_eq!(staged, ["Call the dentist", "Pay rent now"]);
    let status = owner(&sb, &["status"], "status.schema.json");
    assert_eq!(status["waiting"][0]["hash"], retitle.as_str());
    let staged_dentist = status["staged"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["oid"] == dentist.as_str())
        .unwrap();
    assert_eq!(staged_dentist["from"], "vpt");
}

#[test]
fn resolve_settles_a_clash_with_the_items_values_or_the_owners() {
    let _guard = support::guard("resolve_settles_a_clash_with_the_items_values_or_the_owners");
    let (sb, held, rent) = seeded_with_rent();
    let retitle = send(&sb, &held, &["edit", &rent, "--subject", "Pay the rent"]);
    let (ok, _, err) = sb.dam(&["edit", &rent, "--subject", "Pay rent now"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["resolve", &retitle[..7], "--theirs"]);
    assert!(ok, "{err}");
    assert_eq!(out.trim(), format!("{} resolved with theirs", &rent[..7]));
    let status = owner(&sb, &["status"], "status.schema.json");
    assert_eq!(status["staged"][0]["subject"], "Pay the rent");
    assert_eq!(status["staged"][0]["from"], "vpt");
    assert_eq!(status["waiting"], serde_json::json!([]));
}
