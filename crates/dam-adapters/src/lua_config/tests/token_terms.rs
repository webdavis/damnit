use dam_domain::{Channel, Path, Permission};

use super::support::{only_disabled_reason, only_terms, token, tokens, value};

#[test]
fn a_token_table_reads_its_value_conditions_and_grants() {
    let text = token(
        r#"expires = "2027-05-11T00:00:00Z", not_before = "2026-11-11T00:00:00Z",
        via = { "local", "mcp" },
        grant = {
          { allow = { "task:read", "task:create" }, paths = { "work" }, labels = { "agent" }, own = true },
          { allow = { "remote:pull" }, remotes = { "todoist" } },
        }"#,
    );
    let (entries, warnings) = tokens(&text.replacen("return {", "return { schema = 1,", 1));
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
fn a_token_with_no_via_is_local_only_and_requires_nothing() {
    let terms = only_terms(&token(""));
    assert_eq!(terms.via, [Channel::Local].into());
    assert!(terms.grants.is_empty());
    assert!(terms.require.is_empty());
}

#[test]
fn require_names_the_request_details_the_token_must_send() {
    let terms = only_terms(&token("require = { 'session', 'model' }"));
    assert_eq!(terms.require, ["session", "model"]);
    only_disabled_reason(&token("require = 'session'"));
}

#[test]
fn an_unknown_permission_is_skipped_with_a_warning_naming_it() {
    let text = token("grant = { { allow = { 'task:read', 'task:teleport' } } }");
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
    let why = only_disabled_reason(&token(
        "grant = { { allow = { 'task:read', 'config:change' } } }",
    ));
    assert!(why.contains("config:change"), "{why}");
}

#[test]
fn an_unknown_key_in_a_token_or_grant_disables_the_token_and_names_the_key() {
    let in_token = only_disabled_reason(&token("except_paths = { 'home' }"));
    assert!(
        in_token.contains("except_paths") && in_token.contains("token.tool of secrets.lua"),
        "{in_token}"
    );
    let in_grant = only_disabled_reason(&token("grant = { { allow = {}, weekdays = { 'mon' } } }"));
    assert!(
        in_grant.contains("weekdays") && in_grant.contains("token.tool.grant"),
        "{in_grant}"
    );
}

#[test]
fn a_malformed_condition_or_restriction_disables_the_token() {
    for body in [
        "expires = '2027-05-11T00:00:00'",
        "expires = 20270511",
        "via = 'local'",
        "limits = 5",
        "grant = { { paths = { 'work' } } }",
        "grant = { { allow = {}, paths = { 'a//b' } } }",
        "grant = { { allow = {}, own = 'yes' } }",
        "grant = { allow = {} }",
    ] {
        only_disabled_reason(&token(body));
    }
}

#[test]
fn an_unknown_channel_is_skipped_with_a_warning() {
    let text = token("via = { 'local', 'carrier-pigeon' }");
    assert_eq!(only_terms(&text).via, [Channel::Local].into());
    assert!(tokens(&text).1[0].contains("carrier-pigeon"));
}
