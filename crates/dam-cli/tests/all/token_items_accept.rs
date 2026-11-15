use crate::support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

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
        "tool",
        1,
        vec![grant(&["task:read", "task:create", "task:change"])],
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
    assert_eq!(added["staged"][0]["from"], "tool");
    assert_eq!(added["staged"][1]["subject"], "Pay the rent");
    let commit = owner(&sb, &["commit", "-m", "tool work"], "commit.schema.json");
    assert_eq!(commit["changes"][0]["from"], "tool");
    assert_eq!(commit["changes"][0]["channel"], "local");

    let log = owner(&sb, &["log"], "log.schema.json");
    assert_eq!(log["commits"][0]["changes"][1]["from"], "tool");
    assert_eq!(
        log["commits"][1]["changes"][0]["from"],
        serde_json::Value::Null
    );
    let (_, text, _) = sb.dam(&["log"]);
    assert!(
        text.lines()
            .next()
            .unwrap()
            .ends_with("2 changes  tool  tool work"),
        "{text}"
    );
    let (_, shown, _) = sb.dam(&["show", &dentist]);
    assert!(shown.contains("Call the dentist"), "{shown}");
    let id = commit["id"].as_str().unwrap();
    let (_, shown, _) = sb.dam(&["show", &id[..7]]);
    assert!(shown.contains("Call the dentist  tool"), "{shown}");
}
