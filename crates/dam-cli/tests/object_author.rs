mod support;

use serde_json::Value;
use support::sandbox::Sandbox;
use support::tokens::{answer, grant_to};

fn owned_by_stephen() -> Sandbox {
    let sb = Sandbox::new();
    let path = sb.dir.path().join("config.toml");
    let config = std::fs::read_to_string(&path).unwrap();
    std::fs::write(path, format!("owner = \"Stephen\"\n{config}")).unwrap();
    sb
}

fn agent(sb: &Sandbox, name: &str, fill: u8) -> String {
    grant_to(
        sb,
        name,
        fill,
        &format!("[[token.{name}.grant]]\nallow = [\"task:read\", \"task:create\"]\n"),
    )
}

fn accepted(sb: &Sandbox, held: &str, subject: &str, session: &str) -> String {
    let meta = format!("session={session}");
    let out = sb.output_holding(held, &["new", subject, "--json", "--meta", &meta]);
    let hash = answer(&out, "sent.schema.json")["hash"]
        .as_str()
        .unwrap()
        .to_string();
    let (ok, _, err) = sb.dam(&["add", &hash]);
    assert!(ok, "{err}");
    hash
}

fn shown(sb: &Sandbox, held: Option<&str>, oid: &str) -> Value {
    let args = ["show", oid, "--json", "--no-pull"];
    let out = match held {
        Some(held) => sb.output_holding(held, &args),
        None => sb.output(&args),
    };
    answer(&out, "show.schema.json")
}

#[test]
fn an_object_names_its_author_and_the_details_of_the_request_that_made_it() {
    let _guard =
        support::guard("an_object_names_its_author_and_the_details_of_the_request_that_made_it");
    let sb = owned_by_stephen();
    let claude = agent(&sb, "claude", 1);
    let dentist = accepted(&sb, &claude, "Call the dentist", "s1");
    let (ok, _, err) = sb.dam(&["commit", "-m", "dentist"]);
    assert!(ok, "{err}");
    let rent = sb.new_object(&["Pay rent"]);
    let theirs = shown(&sb, None, &dentist);
    assert_eq!(theirs["author"], "claude");
    assert_eq!(theirs["details"]["meta"]["session"], "s1");
    let mine = shown(&sb, None, &rent);
    assert_eq!(mine["author"], "Stephen");
    assert!(mine["details"].is_null(), "{mine}");
    let (ok, human, err) = sb.dam(&["show", &dentist, "--no-pull"]);
    assert!(ok, "{err}");
    assert!(human.contains("author: claude"), "{human}");
    assert!(human.contains("session: s1"), "{human}");
    let (ok, toon, err) = sb.dam(&["show", &dentist, "--toon", "--no-pull"]);
    assert!(ok, "{err}");
    assert!(
        toon.contains("author: claude") && toon.contains("session: s1"),
        "{toon}"
    );
}

#[test]
fn a_token_reads_the_details_of_its_own_objects_only() {
    let _guard = support::guard("a_token_reads_the_details_of_its_own_objects_only");
    let sb = owned_by_stephen();
    let claude = agent(&sb, "claude", 1);
    let codex = agent(&sb, "codex", 2);
    let dentist = accepted(&sb, &claude, "Call the dentist", "s1");
    let own = shown(&sb, Some(&claude), &dentist);
    assert_eq!(own["details"]["meta"]["session"], "s1");
    let other = shown(&sb, Some(&codex), &dentist);
    assert_eq!(other["author"], "claude");
    assert!(other["details"].is_null(), "{other}");
}

#[test]
fn ls_shows_the_author_and_filters_by_mine_agents_or_all() {
    let _guard = support::guard("ls_shows_the_author_and_filters_by_mine_agents_or_all");
    let sb = owned_by_stephen();
    let claude = agent(&sb, "claude", 1);
    accepted(&sb, &claude, "Call the dentist", "s1");
    sb.new_object(&["Pay rent"]);
    let subjects = |by: &str, format: &str| -> Vec<String> {
        let out = sb.output(&["ls", "--no-pull", "--by", by, format]);
        assert!(out.status.success(), "{out:?}");
        let text = String::from_utf8(out.stdout).unwrap();
        if format == "--toon" {
            return ["Call the dentist", "Pay rent"]
                .into_iter()
                .filter(|s| text.contains(s))
                .map(String::from)
                .collect();
        }
        let listed: Value = serde_json::from_str(&text).unwrap();
        support::schema::assert_matches_schema(&listed, "ls.schema.json");
        listed["objects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| format!("{} by {}", o["subject"].as_str().unwrap(), o["author"]))
            .collect()
    };
    assert_eq!(subjects("mine", "--json"), ["Pay rent by \"Stephen\""]);
    assert_eq!(
        subjects("agents", "--json"),
        ["Call the dentist by \"claude\""]
    );
    assert_eq!(subjects("all", "--json").len(), 2);
    assert_eq!(subjects("agents", "--toon"), ["Call the dentist"]);
    assert_eq!(subjects("mine", "--toon"), ["Pay rent"]);
    let (ok, human, err) = sb.dam(&["ls", "--no-pull"]);
    assert!(ok, "{err}");
    let dentist = human
        .lines()
        .find(|l| l.contains("Call the dentist"))
        .unwrap();
    assert!(dentist.contains("claude"), "{human}");
    let rent = human.lines().find(|l| l.contains("Pay rent")).unwrap();
    assert!(rent.contains("Stephen"), "{human}");
}
