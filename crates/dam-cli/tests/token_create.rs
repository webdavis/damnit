mod support;

use std::os::unix::fs::PermissionsExt;

use dam_domain::Token;
use support::errors::{error_document, kind_of};
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn config_of(sb: &Sandbox) -> String {
    std::fs::read_to_string(sb.dir.path().join("config.toml")).unwrap()
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

#[test]
fn a_created_token_lets_its_holder_run_what_the_grant_allows() {
    let _guard = support::guard("a_created_token_lets_its_holder_run_what_the_grant_allows");
    let sb = Sandbox::new();
    let before = config_of(&sb);
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
    let after = config_of(&sb);
    assert!(after.starts_with(&before), "{after}");
    let mode = std::fs::metadata(sb.dir.path().join("config.toml"))
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
    let config = sb.dir.path().join("config.toml");
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
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
    let before = config_of(&sb);
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
    assert_eq!(config_of(&sb), before);
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
    assert!(config_of(&sb).contains("own = true\n"));
    let value = answer["value"].as_str().unwrap();
    let out = sb.output_holding(value, &["ls", "--json", "--no-pull"]);
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(listed["objects"], serde_json::json!([]));
}
