mod support;

use std::os::unix::fs::PermissionsExt;

use dam_domain::Token;
use support::errors::{error_document, kind_of};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn secrets_of(sb: &Sandbox) -> String {
    std::fs::read_to_string(sb.secrets_path()).unwrap()
}

fn created(sb: &Sandbox, args: &[&str]) -> serde_json::Value {
    let mut line = vec!["token", "create"];
    line.extend_from_slice(args);
    line.push("--json");
    let (ok, out, err) = sb.dam(&line);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_matches_schema(&answer, "token-create.schema.json");
    answer
}

fn inserted_block(before: &str, after: &str) -> String {
    let before: Vec<&str> = before.lines().collect();
    let after: Vec<&str> = after.lines().collect();
    let head = before
        .iter()
        .zip(&after)
        .take_while(|(b, a)| b == a)
        .count();
    let tail = before[head..]
        .iter()
        .rev()
        .zip(after[head..].iter().rev())
        .take_while(|(b, a)| b == a)
        .count();
    assert_eq!(
        head + tail,
        before.len(),
        "the old file is not the new file minus one inserted block:\n{}",
        after.join("\n")
    );
    after[head..after.len() - tail].join("\n")
}

#[test]
fn a_created_token_lets_its_holder_run_what_the_grant_allows() {
    let _guard = support::guard("a_created_token_lets_its_holder_run_what_the_grant_allows");
    let sb = Sandbox::new();
    let before = secrets_of(&sb);
    let answer = created(
        &sb,
        &[
            "tool",
            "--allow",
            "task:read",
            "--allow",
            "event:read",
            "--paths",
            "work",
            "--expires",
            "2999-05-11T00:00:00Z",
        ],
    );
    let value = answer["value"].as_str().unwrap();
    let token = Token::parse(value).unwrap();
    assert_eq!(answer["name"], "tool");
    assert_eq!(answer["key_id"], token.key_id().as_str());
    assert_eq!(answer["expires"], "2999-05-11T00:00:00Z");
    assert_eq!(answer["via"], serde_json::json!(["local"]));
    assert_eq!(
        answer["grants"],
        serde_json::json!([{"allow": ["task:read", "event:read"], "paths": ["work/"],
                            "labels": null, "own": false, "remotes": null}])
    );
    let after = secrets_of(&sb);
    let inserted = inserted_block(&before, &after);
    assert!(inserted.contains("tool"), "{inserted}");
    let mode = std::fs::metadata(sb.secrets_path())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
    let out = sb.output_holding(value, &["ls", "--json", "--no-pull"]);
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn text_prints_the_token_alone_and_toon_carries_the_json_facts() {
    let _guard = support::guard("text_prints_the_token_alone_and_toon_carries_the_json_facts");
    let sb = Sandbox::new();
    std::fs::set_permissions(sb.secrets_path(), std::fs::Permissions::from_mode(0o600)).unwrap();
    let (ok, out, err) = sb.dam(&["token", "create", "a", "--allow", "task:read"]);
    assert!(ok, "{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(Token::parse(out.trim_end()).is_ok(), "{out}");
    let (ok, out, err) = sb.dam(&[
        "token",
        "create",
        "b",
        "--allow",
        "task:read",
        "--via",
        "mcp",
        "--toon",
    ]);
    assert!(ok, "{err}");
    for fact in ["name: b", "value: damk_v1_", "key_id: ", "via[1]: mcp"] {
        assert!(out.contains(fact), "{fact} missing from {out}");
    }
}

#[test]
fn a_taken_name_a_bad_name_and_a_bad_permission_change_nothing() {
    let _guard = support::guard("a_taken_name_a_bad_name_and_a_bad_permission_change_nothing");
    let sb = Sandbox::new();
    created(&sb, &["tool", "--allow", "task:read"]);
    let before = secrets_of(&sb);
    let cases: [(&[&str], i32, &str, Option<&str>); 5] = [
        (
            &["tool", "--allow", "task:read"],
            4,
            "refused",
            Some("token_exists"),
        ),
        (&["v.p", "--allow", "task:read"], 2, "usage", None),
        (&["x", "--allow", "task:fly"], 2, "usage", None),
        (
            &["x", "--allow", "config:change"],
            4,
            "refused",
            Some("not_grantable"),
        ),
        (
            &["x", "--allow", "task:read", "--expires", "soon"],
            2,
            "usage",
            None,
        ),
    ];
    for (args, code, kind, rule) in cases {
        let mut line = vec!["token", "create"];
        line.extend_from_slice(args);
        line.push("--json");
        let out = sb.output(&line);
        assert_eq!(out.status.code(), Some(code), "{args:?}: {out:?}");
        assert_matches_schema(&error_document(&out), "error.schema.json");
        assert_eq!(kind_of(&out), kind, "{args:?}");
        assert_eq!(error_document(&out)["error"]["rule"].as_str(), rule);
    }
    let taken = sb.output(&["token", "create", "tool", "--allow", "task:read"]);
    assert_eq!(
        String::from_utf8_lossy(&taken.stderr),
        "dam: token tool already exists\n"
    );
    assert_eq!(secrets_of(&sb), before);
}

#[test]
fn a_reserved_name_cannot_be_created_for_any_token() {
    let _guard = support::guard("a_reserved_name_cannot_be_created_for_any_token");
    let sb = Sandbox::new();
    for reserved in ["token:create", "token:revoke", "history:rewrite"] {
        let out = sb.output(&["token", "create", "x", "--allow", reserved, "--json"]);
        assert_eq!(out.status.code(), Some(4), "{out:?}");
        assert!(
            error_document(&out)["error"]["message"]
                .as_str()
                .unwrap()
                .contains(reserved)
        );
    }
}

#[test]
fn own_narrows_the_grant_to_what_the_token_made() {
    let _guard = support::guard("own_narrows_the_grant_to_what_the_token_made");
    let sb = Sandbox::new();
    sb.new_object(&["diary"]);
    let answer = created(&sb, &["tool", "--allow", "task:read", "--own"]);
    assert_eq!(answer["grants"][0]["own"], true);
    assert!(secrets_of(&sb).contains("own = true"));
    let value = answer["value"].as_str().unwrap();
    let out = sb.output_holding(value, &["ls", "--json", "--no-pull"]);
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(listed["objects"], serde_json::json!([]));
}
