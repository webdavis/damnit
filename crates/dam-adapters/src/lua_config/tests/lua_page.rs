use std::collections::BTreeSet;

use dam_domain::{
    Author, Authored, Change, Channel, CommitId, CommitRecord, Details, Object, Timestamp,
    TokenAuthor,
};

use super::filter_fields::NOT_IN_THE_TABLE;
use super::filter_support::{every_field_event, every_field_task, field_names};
use crate::lua_config::{filter, remote, secrets, settings, token};
use crate::{history_document, object_document};

const PAGE: &str = include_str!("../../../../../docs/lua.md");

fn rows_under(heading: &str) -> BTreeSet<String> {
    PAGE.split("\n## ")
        .find(|section| section.starts_with(heading))
        .unwrap_or_else(|| panic!("docs/lua.md has no section {heading}"))
        .lines()
        .filter_map(|line| line.strip_prefix("| `")?.split_once('`'))
        .map(|(name, _)| name.to_string())
        .collect()
}

fn keys_named(rows: &BTreeSet<String>) -> BTreeSet<String> {
    rows.iter()
        .flat_map(|row| row.split('.'))
        .filter(|segment| !segment.starts_with('<'))
        .map(|segment| segment.trim_end_matches("[]").to_string())
        .collect()
}

fn owned(keys: &[&[&str]]) -> BTreeSet<String> {
    keys.iter()
        .flat_map(|k| k.iter())
        .map(|k| k.to_string())
        .collect()
}

#[test]
fn the_page_names_every_key_config_lua_takes_and_no_other() {
    let read = owned(&[
        &settings::SETTINGS[..],
        &settings::DONE[..],
        &settings::CATEGORY[..],
        &remote::SETTINGS[..],
        &filter::FILTER[..],
    ]);
    assert_eq!(keys_named(&rows_under("config.lua")), read);
}

#[test]
fn the_page_names_every_key_secrets_lua_takes_and_no_other() {
    let read = owned(&[
        &secrets::SECRETS[..],
        &token::terms::TOKEN_KEYS[..],
        &token::grant::GRANT_KEYS[..],
    ]);
    assert_eq!(keys_named(&rows_under("secrets.lua")), read);
}

#[test]
fn the_page_names_every_field_match_receives_and_no_other() {
    let handed: BTreeSet<String> = [every_field_task(), every_field_event()]
        .iter()
        .flat_map(|o| field_names(&serde_json::to_value(object_document(o)).unwrap()))
        .collect();
    assert_eq!(rows_under("What match receives"), handed);
    let [first, second, third] = NOT_IN_THE_TABLE;
    let left_out = format!("except `{first}`, `{second}` and `{third}`");
    assert!(PAGE.contains(&left_out), "docs/lua.md must say {left_out}");
}

fn change_by_a_token(object: Object) -> Authored {
    Authored {
        change: Change::Create { after: object },
        author: Author::Token(TokenAuthor {
            name: "tool".into(),
            key_id: dam_domain::Token::generate(&mut |b: &mut [u8]| b.fill(1))
                .key_id()
                .clone(),
            channel: Channel::Local,
        }),
        details: Some(Details {
            meta: Default::default(),
            host: Some("host".into()),
            dir: Some("/".into()),
            repo: Some("repo".into()),
            branch: Some("main".into()),
            at: Timestamp::UNIX_EPOCH,
            channel: Channel::Local,
        }),
    }
}

#[test]
fn the_page_names_every_field_dam_history_returns_and_no_other() {
    let commits = vec![CommitRecord {
        id: CommitId::generate(&mut |b: &mut [u8]| b.fill(9)),
        message: "both".into(),
        at: Timestamp::UNIX_EPOCH,
        changes: vec![
            change_by_a_token(every_field_task()),
            change_by_a_token(every_field_event()),
        ],
    }];
    let returned: BTreeSet<String> = history_document(&commits, "owner")
        .as_array()
        .unwrap()
        .iter()
        .flat_map(field_names)
        .collect();
    assert_eq!(rows_under("What dam.history returns"), returned);
}

fn example() -> &'static str {
    let (_, rest) = PAGE.split_once("```lua\n").unwrap();
    rest.split_once("```").unwrap().0
}

#[test]
fn the_pages_example_reads_as_a_function_filter() {
    let config = super::support::config(example()).load().unwrap();
    assert!(config.filter("postponed").unwrap().rule.runs_a_function());
}
