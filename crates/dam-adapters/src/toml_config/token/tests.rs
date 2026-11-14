use dam_application::{Standing, TokenConfig, TokenTerms};
use dam_domain::{Channel, Path, Permission, Token};

use crate::toml_config::parse_config;

pub(super) fn value(fill: u8) -> String {
    Token::generate(&mut |b: &mut [u8]| b.fill(fill)).expose()
}

pub(super) fn tokens(text: &str) -> (Vec<TokenConfig>, Vec<String>) {
    let tokens = parse_config(text).unwrap().tokens;
    (tokens.entries, tokens.warnings)
}

fn only_terms(text: &str) -> TokenTerms {
    let (entries, _) = tokens(text);
    match &entries[..] {
        [
            TokenConfig {
                standing: Standing::Usable(terms),
                ..
            },
        ] => terms.clone(),
        other => panic!("expected one usable token, got {other:?}"),
    }
}

fn only_disabled_reason(text: &str) -> String {
    let (entries, _) = tokens(text);
    match &entries[..] {
        [
            TokenConfig {
                standing: Standing::Disabled(why),
                ..
            },
        ] => why.clone(),
        other => panic!("expected one disabled token, got {other:?}"),
    }
}

#[test]
fn a_token_table_reads_its_value_conditions_and_grants() {
    let text = format!(
        r#"
schema = 1

[token.tool]
value = "{}"
expires = 2027-05-11T00:00:00Z
not_before = 2026-11-11T00:00:00Z
via = ["local", "mcp"]

[[token.tool.grant]]
allow = ["task:read", "task:create"]
paths = ["work"]
labels = ["agent"]
own = true

[[token.tool.grant]]
allow = ["remote:pull"]
remotes = ["todoist"]
"#,
        value(1)
    );
    let (entries, warnings) = tokens(&text);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(entries[0].name, "tool");
    assert_eq!(entries[0].value.expose(), value(1));
    let terms = only_terms(&text);
    assert_eq!(terms.expires.unwrap().to_string(), "2027-05-11T00:00:00Z");
    assert_eq!(
        terms.not_before.unwrap().to_string(),
        "2026-11-11T00:00:00Z"
    );
    assert_eq!(terms.via, [Channel::Local, Channel::Mcp].into());
    let work = &terms.grants[0];
    assert_eq!(
        work.allow,
        [Permission::TaskRead, Permission::TaskCreate].into()
    );
    assert_eq!(work.paths, Some(vec![Path::parse("work").unwrap()]));
    assert_eq!(work.labels, Some(["agent".to_string()].into()));
    assert!(work.own);
    assert_eq!(terms.grants[1].remotes, Some(vec!["todoist".to_string()]));
}

#[test]
fn a_token_with_no_via_is_local_only() {
    let terms = only_terms(&format!("[token.tool]\nvalue = \"{}\"\n", value(1)));
    assert_eq!(terms.via, [Channel::Local].into());
    assert!(terms.grants.is_empty());
}

#[test]
fn require_names_the_request_details_the_token_must_send() {
    let text = format!(
        "[token.claude]\nvalue = \"{}\"\nrequire = [\"session\", \"model\"]\n",
        value(1)
    );
    assert_eq!(only_terms(&text).require, ["session", "model"]);
    assert!(
        only_terms(&format!("[token.a]\nvalue = \"{}\"\n", value(1)))
            .require
            .is_empty()
    );
    only_disabled_reason(&format!(
        "[token.a]\nvalue = \"{}\"\nrequire = \"session\"\n",
        value(1)
    ));
}

#[test]
fn an_unknown_permission_is_skipped_with_a_warning_naming_it() {
    let text = format!(
        "[token.tool]\nvalue = \"{}\"\n[[token.tool.grant]]\nallow = [\"task:read\", \"task:teleport\"]\n",
        value(1)
    );
    let (_, warnings) = tokens(&text);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("task:teleport") && warnings[0].contains("tool"));
    assert_eq!(
        only_terms(&text).grants[0].allow,
        [Permission::TaskRead].into()
    );
}

#[test]
fn a_reserved_permission_disables_the_token_and_names_it() {
    let why = only_disabled_reason(&format!(
        "[token.tool]\nvalue = \"{}\"\n[[token.tool.grant]]\nallow = [\"task:read\", \"config:change\"]\n",
        value(1)
    ));
    assert!(why.contains("config:change"), "{why}");
}

#[test]
fn an_unknown_key_in_a_token_or_grant_disables_the_token_and_names_the_key() {
    let in_token = only_disabled_reason(&format!(
        "[token.tool]\nvalue = \"{}\"\nexcept_paths = [\"home\"]\n",
        value(1)
    ));
    assert!(in_token.contains("except_paths") && in_token.contains("[token.tool]"));
    let in_grant = only_disabled_reason(&format!(
        "[token.tool]\nvalue = \"{}\"\n[[token.tool.grant]]\nallow = []\nweekdays = [\"mon\"]\n",
        value(1)
    ));
    assert!(in_grant.contains("weekdays") && in_grant.contains("[token.tool.grant]"));
}

#[test]
fn a_malformed_condition_or_restriction_disables_the_token() {
    for body in [
        "expires = 2027-05-11T00:00:00",
        "via = \"local\"",
        "limits = { creates_per_day = -1 }",
        "[[token.tool.grant]]\npaths = [\"work\"]",
        "[[token.tool.grant]]\nallow = []\npaths = [\"a//b\"]",
        "[[token.tool.grant]]\nallow = []\nown = \"yes\"",
    ] {
        only_disabled_reason(&format!("[token.tool]\nvalue = \"{}\"\n{body}\n", value(1)));
    }
}

#[test]
fn an_unknown_channel_is_skipped_with_a_warning() {
    let text = format!(
        "[token.tool]\nvalue = \"{}\"\nvia = [\"local\", \"carrier-pigeon\"]\n",
        value(1)
    );
    assert_eq!(only_terms(&text).via, [Channel::Local].into());
    assert!(tokens(&text).1[0].contains("carrier-pigeon"));
}
