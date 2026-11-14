use super::hidden::Hidden;
use super::sandbox::Sandbox;
use super::token_table::Grant;
use super::tokens::{answer, grant, grant_to};

pub fn own_grant() -> Grant {
    grant(&["task:read", "task:change", "task:create", "task:complete"]).own()
}

pub fn send(sb: &Sandbox, held: &str, args: &[&str]) -> serde_json::Value {
    let mut line = args.to_vec();
    line.push("--json");
    answer(&sb.output_holding(held, &line), "sent.schema.json")
}

pub fn adopt(sb: &Sandbox, hash: &str) {
    for args in [vec!["add", hash], vec!["commit", "-m", "adopt"]] {
        let (ok, _, err) = sb.dam(&args);
        assert!(ok, "{args:?}: {err}");
    }
}

pub fn made_by(sb: &Sandbox, held: &str, subject: &str, path: &str) -> String {
    let sent = send(sb, held, &["new", subject, "--path", path]);
    let hash = sent["hash"].as_str().unwrap().to_string();
    adopt(sb, &hash);
    hash
}

pub fn own_hidden(grant: Grant) -> Hidden {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant]);
    let report = made_by(&sb, &held, "report", "work");
    Hidden { sb, held, report }
}

pub fn listed(h: &Hidden, args: &[&str]) -> Vec<String> {
    let mut line = vec!["ls", "--json", "--no-pull"];
    line.extend_from_slice(args);
    let out = h.holding(&line);
    assert!(out.status.success(), "{out:?}");
    let document: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    document["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["oid"].as_str().unwrap().to_string())
        .collect()
}
