use dam_application::{Expected, Misread, TokenFault, TokenWarning};
use dam_domain::{Channel, Path, PathError, Permission};

use super::support::{in_table, only_disabled_reason, only_terms, token, tokens, value};

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
    assert_eq!(
        warnings,
        [TokenWarning::UnknownPermission {
            name: "tool".into(),
            word: "task:teleport".into(),
        }]
    );
    assert_eq!(
        only_terms(&text).grants[0].allow,
        [Permission::TaskRead].into()
    );
}

#[test]
fn a_reserved_permission_disables_the_token_and_names_it() {
    assert_eq!(
        only_disabled_reason(&token(
            "grant = { { allow = { 'task:read', 'config:change' } } }",
        )),
        TokenFault::Reserved("config:change".into())
    );
}

#[test]
fn an_unknown_key_in_a_token_or_grant_disables_the_token_and_names_the_key() {
    assert_eq!(
        only_disabled_reason(&token("except_paths = { 'home' }")),
        Misread::UnknownKey {
            key: "except_paths".into(),
            at: in_table("token.tool", "secrets.lua"),
        }
        .into()
    );
    assert_eq!(
        only_disabled_reason(&token("grant = { { allow = {}, weekdays = { 'mon' } } }")),
        Misread::UnknownKey {
            key: "weekdays".into(),
            at: in_table("token.tool.grant", "secrets.lua"),
        }
        .into()
    );
}

#[test]
fn a_malformed_condition_or_restriction_disables_the_token() {
    let wrong = |key: &str, table: &str, expected| Misread::WrongType {
        key: key.into(),
        at: in_table(table, "secrets.lua"),
        expected,
    };
    for (body, misread) in [
        (
            "expires = '2027-05-11T00:00:00'",
            wrong("expires", "token.tool", Expected::DateTime),
        ),
        (
            "expires = 20270511",
            wrong("expires", "token.tool", Expected::DateTime),
        ),
        (
            "via = 'local'",
            wrong("via", "token.tool", Expected::ListOfTexts),
        ),
        (
            "limits = 5",
            wrong("limits", "token.tool", Expected::TableOfWholeNumbers),
        ),
        (
            "grant = { { paths = { 'work' } } }",
            Misread::Missing {
                key: "allow".into(),
                at: in_table("token.tool.grant", "secrets.lua"),
            },
        ),
        (
            "grant = { { allow = {}, paths = { 'a//b' } } }",
            Misread::NotAPath {
                text: "a//b".into(),
                at: in_table("token.tool.grant", "secrets.lua"),
                why: PathError::EmptySegment,
            },
        ),
        (
            "grant = { { allow = {}, own = 'yes' } }",
            wrong("own", "token.tool.grant", Expected::TrueOrFalse),
        ),
        (
            "grant = { allow = {} }",
            wrong("grant", "token.tool", Expected::ListOfTables),
        ),
    ] {
        assert_eq!(only_disabled_reason(&token(body)), misread.into(), "{body}");
    }
}

#[test]
fn an_unknown_channel_is_skipped_with_a_warning() {
    let text = token("via = { 'local', 'carrier-pigeon' }");
    assert_eq!(only_terms(&text).via, [Channel::Local].into());
    assert_eq!(
        tokens(&text).1,
        [TokenWarning::UnknownChannel {
            name: "tool".into(),
            word: "carrier-pigeon".into(),
        }]
    );
}
