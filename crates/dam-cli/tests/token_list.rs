mod support;

use dam_domain::Token;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn token(fill: u8) -> Token {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill))
}

fn sandbox_with_four_tokens() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_config(&format!(
        "[token.tool]\nvalue = \"{}\"\nexpires = 2999-01-01T00:00:00Z\nvia = [\"local\", \"mcp\"]\n\
         [[token.tool.grant]]\nallow = [\"task:read\"]\npaths = [\"work\"]\n\
         [token.old]\nvalue = \"{}\"\nexpires = 2000-01-01T00:00:00Z\n\
         [token.odd]\nvalue = \"{}\"\nweekdays = [\"mon\"]\n\
         [token.bare]\n",
        token(1).expose(),
        token(2).expose(),
        token(3).expose()
    ));
    sb
}

#[test]
fn the_list_names_each_token_its_key_id_terms_and_standing() {
    let _guard = support::guard("the_list_names_each_token_its_key_id_terms_and_standing");
    let sb = sandbox_with_four_tokens();
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_matches_schema(&answer, "token-list.schema.json");
    let named = |name: &str| {
        answer["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .unwrap()
            .clone()
    };
    let tool = named("tool");
    assert_eq!(tool["name"], "tool");
    assert_eq!(tool["key_id"], token(1).key_id().as_str());
    assert_eq!(tool["standing"], "active");
    assert_eq!(tool["expires"], "2999-01-01T00:00:00Z");
    assert_eq!(tool["via"], serde_json::json!(["local", "mcp"]));
    assert_eq!(tool["grants"][0]["allow"], serde_json::json!(["task:read"]));
    assert_eq!(tool["grants"][0]["paths"], serde_json::json!(["work/"]));
    assert_eq!(named("old")["standing"], "expired");
    let odd = named("odd");
    assert_eq!(odd["standing"], "disabled");
    assert!(odd["why"].as_str().unwrap().contains("weekdays"), "{odd}");
    assert_eq!(answer["tokens"].as_array().unwrap().len(), 3);
    assert!(answer["warnings"][0].as_str().unwrap().contains("bare"));
}

#[test]
fn no_format_prints_a_secret_and_each_carries_the_same_tokens() {
    let _guard = support::guard("no_format_prints_a_secret_and_each_carries_the_same_tokens");
    let sb = sandbox_with_four_tokens();
    for format in [None, Some("--json"), Some("--toon")] {
        let mut line = vec!["token", "list"];
        line.extend(format);
        let out = sb.command(&line).env("COLUMNS", "300").output().unwrap();
        assert!(out.status.success(), "{out:?}");
        let out = String::from_utf8(out.stdout).unwrap();
        for fill in 1..=3 {
            let held = token(fill);
            assert!(!out.contains(&held.expose()), "{format:?} leaked {out}");
            assert!(out.contains(held.key_id().as_str()), "{format:?}: {out}");
        }
        for fact in [
            "tool",
            "active",
            "expired",
            "disabled",
            "weekdays",
            "task:read",
        ] {
            assert!(out.contains(fact), "{format:?} lacks {fact}: {out}");
        }
    }
}

#[test]
fn a_config_without_tokens_lists_none() {
    let _guard = support::guard("a_config_without_tokens_lists_none");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(answer, serde_json::json!({"tokens": [], "warnings": []}));
    let (_, out, _) = sb.dam(&["token", "list"]);
    assert_eq!(out, "no tokens declared\n");
}

#[test]
fn a_token_that_requires_details_lists_as_active_with_what_it_requires() {
    let _guard =
        support::guard("a_token_that_requires_details_lists_as_active_with_what_it_requires");
    let sb = Sandbox::new();
    sb.add_config(&format!(
        "\n[token.asks]\nvalue = \"{}\"\nrequire = [\"ticket\"]\n\
         [[token.asks.grant]]\nallow = [\"task:read\"]\n",
        token(4).expose()
    ));
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    let asks = &answer["tokens"][0];
    assert_eq!(asks["standing"], "active", "{asks}");
    assert_eq!(asks["require"], serde_json::json!(["ticket"]));
}
