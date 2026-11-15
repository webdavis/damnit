use std::os::unix::fs::PermissionsExt;

use super::sandbox::Sandbox;

pub fn task(remote_id: &str, subject: &str, completed_at: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "oid": "", "remote_id": remote_id, "kind": "task", "subject": subject, "body": "",
        "path": "", "labels": [], "depends": [], "reminders": [], "recurrence": null,
        "task": {"done": completed_at.is_some(), "completed_at": completed_at, "priority": 4,
            "due": null, "deadline": null, "event": null},
        "event": null
    })
}

pub fn answer_pull_with(sb: &Sandbox, answer: serde_json::Value) {
    let answer_file = sb.dir.path().join("pull-answer.json");
    std::fs::write(&answer_file, answer.to_string()).unwrap();
    let script = format!(
        r#"#!/bin/sh
set -eu
while IFS= read -r line; do
  case "$line" in
    *'"capabilities"'*) printf '{{"protocol":1,"kinds":["task"],"fields":["subject","done"],"credentials":["api_token"],"incremental":true}}\n' ;;
    *'"pull"'*) printf '%s\n' "$line" >> "{log}"; cat "{answer}"; printf '\n' ;;
    *) printf '{{"error":"unknown request"}}\n' ;;
  esac
done
"#,
        log = sb.dir.path().join("pulls.jsonl").display(),
        answer = answer_file.display(),
    );
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    std::fs::write(&helper, script).unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
}

pub fn pulls_asked_since(sb: &Sandbox) -> Vec<serde_json::Value> {
    std::fs::read_to_string(sb.dir.path().join("pulls.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap()["since"].clone())
        .collect()
}
