mod support;

use dam_domain::Token;
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::token_table::{Grant, TokenTable};

fn token(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

fn sandbox_granting(allow: &[&str]) -> (Sandbox, String) {
    let sb = Sandbox::new();
    sb.add_token(
        "tool",
        TokenTable::valued(&token(1)).grant(Grant::allowing(allow)),
    );
    (sb, token(1))
}

fn rule_of(out: &std::process::Output) -> String {
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_matches_schema(&error_document(out), "error.schema.json");
    error_document(out)["error"]["rule"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn every_write_verb_is_refused_to_a_token_and_the_store_is_untouched() {
    let _guard =
        support::guard("every_write_verb_is_refused_to_a_token_and_the_store_is_untouched");
    let (sb, held) = sandbox_granting(&["task:read", "task:create", "task:change"]);
    let oid = sb.new_object(&["milk"]);
    for line in [
        vec!["mv", &oid, "work"],
        vec!["add", "-A"],
        vec!["reset"],
        vec!["restore", &oid],
        vec!["commit", "-m", "m"],
        vec!["remote", "add", "other", "fake::"],
        vec!["resolve", &oid, "--ours"],
    ] {
        let mut args = line.clone();
        args.push("--json");
        let out = sb.output_holding(&held, &args);
        assert_eq!(rule_of(&out), "not_open_to_tokens", "{line:?}");
    }
    let (ok, listed, _) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok);
    assert!(listed.contains("milk"));
}

#[test]
fn a_read_the_token_does_not_hold_is_refused_and_one_it_holds_runs() {
    let _guard = support::guard("a_read_the_token_does_not_hold_is_refused_and_one_it_holds_runs");
    let (sb, held) = sandbox_granting(&["config:read"]);
    let out = sb.output_holding(&held, &["log", "--json"]);
    assert_eq!(rule_of(&out), "not_permitted");
    assert!(
        error_document(&out)["error"]["message"]
            .as_str()
            .unwrap()
            .contains("history:read")
    );
    let out = sb.output_holding(&held, &["category", "list", "--json"]);
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn a_token_reaches_only_the_remotes_its_grant_names() {
    let _guard = support::guard("a_token_reaches_only_the_remotes_its_grant_names");
    let sb = Sandbox::new();
    let held = token(1);
    sb.add_token(
        "tool",
        TokenTable::valued(&held)
            .grant(Grant::allowing(&["remote:pull", "remote:push"]).remotes(&["elsewhere"])),
    );
    for verb in ["pull", "push"] {
        for named in ["fake", "nope"] {
            let out = sb.output_holding(&held, &[verb, named, "--json"]);
            assert_eq!(rule_of(&out), "not_permitted", "{verb} {named}");
        }
        let out = sb.output_holding(&held, &[verb, "--json"]);
        assert!(
            out.status.success(),
            "{verb} of every remote narrows to none: {out:?}"
        );
    }
    assert!(sb.pushes().is_empty());
}

#[test]
fn an_owner_only_write_refuses_a_token_it_could_not_look_up() {
    let _guard = support::guard("an_owner_only_write_refuses_a_token_it_could_not_look_up");
    let sb = Sandbox::new();
    let out = sb.output_holding("not-a-token", &["add", "-A", "--json"]);
    assert_eq!(rule_of(&out), "not_open_to_tokens");
}
