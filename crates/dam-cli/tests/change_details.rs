mod support;

use serde_json::Value;
use support::sandbox::Sandbox;
use support::tokens::{answer, grant_to};

fn claude(sb: &Sandbox) -> String {
    grant_to(
        sb,
        "claude",
        1,
        "[[token.claude.grant]]\nallow = [\"task:read\", \"task:create\", \"history:read\"]\n",
    )
}

fn owned_by_stephen(sb: &Sandbox) {
    let path = sb.dir.path().join("config.toml");
    let config = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, format!("owner = \"Stephen\"\n{config}")).unwrap();
}

fn json(out: &std::process::Output) -> Value {
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn sent_by_claude(sb: &Sandbox, held: &str) -> String {
    let out = sb.output_holding(
        held,
        &[
            "new",
            "Call the dentist",
            "--json",
            "--meta",
            "session=s1",
            "--meta",
            "model=opus",
            "--meta",
            "ticket=T-9",
        ],
    );
    answer(&out, "sent.schema.json")["hash"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn a_tokens_change_keeps_the_details_it_stated_and_those_of_the_process() {
    let _guard =
        support::guard("a_tokens_change_keeps_the_details_it_stated_and_those_of_the_process");
    let sb = Sandbox::new();
    let held = claude(&sb);
    sent_by_claude(&sb, &held);
    let status = json(&sb.output(&["status", "--json"]));
    let waiting = &status["waiting"][0];
    assert_eq!(waiting["author"], "claude");
    let details = &waiting["details"];
    assert_eq!(details["meta"]["session"], "s1");
    assert_eq!(details["meta"]["ticket"], "T-9");
    assert_eq!(details["channel"], "local");
    assert!(!details["host"].as_str().unwrap().is_empty(), "{details}");
    let dir = std::fs::canonicalize(sb.dir.path()).unwrap();
    let said = std::fs::canonicalize(details["dir"].as_str().unwrap()).unwrap();
    assert_eq!(said, dir);
    assert!(
        details["repo"].is_null() && details["branch"].is_null(),
        "{details}"
    );
    assert!(details["at"].as_str().unwrap().contains('T'), "{details}");
}

#[test]
fn a_commit_keeps_each_senders_details_and_names_the_owner_from_the_config() {
    let _guard =
        support::guard("a_commit_keeps_each_senders_details_and_names_the_owner_from_the_config");
    let sb = Sandbox::new();
    owned_by_stephen(&sb);
    let held = claude(&sb);
    let hash = sent_by_claude(&sb, &held);
    let mine = sb.new_object(&["Pay rent"]);
    let (ok, _, err) = sb.dam(&["add", &hash, &mine, "--meta", "harness=shell"]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["commit", "-m", "both"]);
    assert!(ok, "{err}");
    let log = json(&sb.output(&["log", "--json"]));
    let changes = log["commits"][0]["changes"].as_array().unwrap();
    let by = |author: &str| changes.iter().find(|c| c["author"] == author).unwrap();
    assert_eq!(by("claude")["details"]["meta"]["session"], "s1");
    assert_eq!(by("Stephen")["details"]["meta"]["harness"], "shell");
}

#[test]
fn the_repo_and_branch_are_those_of_the_working_directory() {
    let _guard = support::guard("the_repo_and_branch_are_those_of_the_working_directory");
    let sb = Sandbox::new();
    let repo = sb.dir.path().join("proj");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/topic\n").unwrap();
    let held = claude(&sb);
    let out = sb
        .command(&["new", "Fix the build", "--json"])
        .current_dir(repo.join("src"))
        .env("DAM_TOKEN", &held)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let status = json(&sb.output(&["status", "--json"]));
    let details = &status["waiting"][0]["details"];
    assert_eq!(details["branch"], "topic");
    let said = std::fs::canonicalize(details["repo"].as_str().unwrap()).unwrap();
    assert_eq!(said, std::fs::canonicalize(&repo).unwrap());
}

#[test]
fn a_token_reads_the_author_of_another_senders_change_but_not_its_details() {
    let _guard =
        support::guard("a_token_reads_the_author_of_another_senders_change_but_not_its_details");
    let sb = Sandbox::new();
    let held = claude(&sb);
    let hash = sent_by_claude(&sb, &held);
    let (ok, _, err) = sb.dam(&["add", &hash]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["commit", "-m", "dentist"]);
    assert!(ok, "{err}");
    let reader = grant_to(
        &sb,
        "auditor",
        2,
        "[[token.auditor.grant]]\nallow = [\"task:read\", \"history:read\"]\n",
    );
    let theirs = json(&sb.output_holding(&reader, &["log", "--json"]));
    let change = &theirs["commits"][0]["changes"][0];
    assert_eq!(change["author"], "claude");
    assert!(change["details"].is_null(), "{change}");
    let own = json(&sb.output_holding(&held, &["log", "--json"]));
    assert_eq!(
        own["commits"][0]["changes"][0]["details"]["meta"]["model"],
        "opus"
    );
}
