mod support;

use serde_json::{Value, json};
use support::config_tokens::config_tokens;
use support::schema::assert_matches_schema;

fn json_of(sb: &support::sandbox::Sandbox, args: &[&str]) -> Value {
    let out = sb.output(&[args, &["--json"]].concat());
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn a_token_lists_each_of_its_settings_by_its_key_in_secrets_lua() {
    let _guard = support::guard("a_token_lists_each_of_its_settings_by_its_key_in_secrets_lua");
    let tokens = config_tokens();
    let (ok, out, err) = tokens.sb.dam(&["config", "get", "token.tool"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "  token.tool.grant       task:read paths work/ | task:create own
  token.tool.expires     2027-05-11T00:00:00Z
  token.tool.not_before  2026-01-01T00:00:00Z
  token.tool.via         mcp
  token.tool.require     session
"
    );
    let (ok, out, err) = tokens.sb.dam(&["config", "get", "token.plain"]);
    assert!(ok, "{err}");
    assert_eq!(
        out,
        "  token.plain.grant       task:read
  token.plain.expires     not set
  token.plain.not_before  not set
  token.plain.via         local (default)
  token.plain.require     not set
"
    );
    let (ok, out, err) = tokens.sb.dam(&["config", "get", "token.off"]);
    assert!(ok, "{err}");
    assert_eq!(out, "  token.off  disabled\n");
}

#[test]
fn a_token_holds_the_same_facts_as_dam_token_list() {
    let _guard = support::guard("a_token_holds_the_same_facts_as_dam_token_list");
    let tokens = config_tokens();
    let listed = json_of(&tokens.sb, &["token", "list"]);
    let settings = json_of(&tokens.sb, &["config", "list"]);
    assert_matches_schema(&settings, "config-list.schema.json");
    for name in ["tool", "plain"] {
        let row = listed["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == name)
            .unwrap();
        let part = &settings["settings"]["token"][name];
        assert_eq!(part["grant"]["value"], row["grants"], "{name}");
        for key in ["expires", "not_before", "via"] {
            assert_eq!(part[key]["value"], row[key], "{name} {key}");
        }
        let require = &part["require"]["value"];
        let require = if require.is_null() {
            json!([])
        } else {
            require.clone()
        };
        assert_eq!(require, row["require"], "{name}");
    }
    let plain = &settings["settings"]["token"]["plain"];
    assert_eq!(plain["via"], json!({"value": ["local"], "default": true}));
    assert_eq!(plain["expires"], json!({"value": null, "default": true}));
    assert_eq!(
        settings["settings"]["token"]["off"],
        json!({"value": "disabled", "default": false})
    );
    let text = settings.to_string();
    for value in &tokens.values {
        assert!(!text.contains(value.as_str()), "{text}");
    }
}

#[test]
fn a_token_with_no_grant_says_so_as_dam_token_list_does() {
    let _guard = support::guard("a_token_with_no_grant_says_so_as_dam_token_list_does");
    let tokens = config_tokens();
    let (ok, out, err) = tokens.sb.dam(&["config", "get", "token.empty.grant"]);
    assert!(ok, "{err}");
    assert_eq!(out, "  token.empty.grant  -\n");
    let (ok, listed, err) = tokens.sb.dam(&["token", "list"]);
    assert!(ok, "{err}");
    let row = listed.lines().find(|l| l.starts_with("empty ")).unwrap();
    assert!(row.contains(" - "), "{row}");
    assert_eq!(
        json_of(&tokens.sb, &["config", "get", "token.empty.grant"]),
        json!({"value": [], "default": false})
    );
}

#[test]
fn a_token_left_with_no_channel_or_permission_reads_a_dash() {
    let _guard = support::guard("a_token_left_with_no_channel_or_permission_reads_a_dash");
    let sb = support::sandbox::Sandbox::new();
    sb.write_secrets_lua(&format!(
        "return {{ schema = 1, token = {{ odd = {{ value = \"{}\", via = {{ \"carrier-pigeon\" }},\n  \
         grant = {{ {{ allow = {{ \"task:juggle\" }} }} }} }} }} }}\n",
        support::tokens::token(5).expose()
    ));
    let (ok, out, err) = sb.dam(&["config", "get", "token.odd"]);
    assert!(ok, "{err}");
    assert!(out.contains("  token.odd.grant       -\n"), "{out}");
    assert!(out.contains("  token.odd.via         -\n"), "{out}");
}

#[test]
fn an_ignored_or_disabled_token_is_warned_of_by_its_name_alone() {
    let _guard = support::guard("an_ignored_or_disabled_token_is_warned_of_by_its_name_alone");
    let tokens = config_tokens();
    let reasoned = tokens.sb.doctor(&["doctor"]).out;
    assert!(reasoned.contains("token stray is ignored: "), "{reasoned}");
    assert!(reasoned.contains("token off is disabled: "), "{reasoned}");
    let warning = "warning: token stray is ignored; run dam doctor to see why\n\
warning: token off is disabled; run dam doctor to see why\n";
    for (line, on_standard_error) in [
        (&["config", "list"][..], warning),
        (&["config", "get", "owner"], warning),
        (&["config", "list", "--json"], ""),
        (&["config", "get", "token", "--toon"], ""),
    ] {
        let out = tokens.sb.output(line);
        assert!(out.status.success(), "{line:?}: {out:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(stderr, on_standard_error, "{line:?}");
        let both = format!("{stderr}{}", String::from_utf8_lossy(&out.stdout));
        assert!(!both.contains("MARKER-stray"), "{line:?}: {both}");
        for value in &tokens.values {
            assert!(!both.contains(value.as_str()), "{line:?}: {both}");
        }
    }
    let listed = json_of(&tokens.sb, &["config", "list"]);
    let in_the_document: Vec<String> = listed["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| format!("warning: {}\n", w.as_str().unwrap()))
        .collect();
    assert_eq!(in_the_document.concat(), warning);
}
