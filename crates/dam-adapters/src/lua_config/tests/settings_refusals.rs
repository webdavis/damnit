use dam_application::{ConfigError, Expected, Misread};

use super::support::{config, in_table, top_of};

#[test]
fn an_unknown_key_is_refused_naming_the_key_and_its_table() {
    assert_eq!(
        config("return { colour = 'red' }").refusal(),
        Misread::UnknownKey {
            key: "colour".into(),
            at: top_of("config.lua"),
        }
        .into()
    );
    assert_eq!(
        config("return { done = { interactive = true, loud = true } }").refusal(),
        Misread::UnknownKey {
            key: "loud".into(),
            at: in_table("done", "config.lua"),
        }
        .into()
    );
}

#[test]
fn a_value_of_the_wrong_type_is_refused_by_name() {
    assert_eq!(
        config("return { remote = { todoist = { url = 'todoist::', stale = 15 } } }").refusal(),
        Misread::WrongType {
            key: "stale".into(),
            at: in_table("remote.todoist", "config.lua"),
            expected: Expected::Text,
        }
        .into()
    );
    assert_eq!(
        config("return { owner = true }").refusal(),
        Misread::WrongType {
            key: "owner".into(),
            at: top_of("config.lua"),
            expected: Expected::Text,
        }
        .into()
    );
}

#[test]
fn tokens_and_the_schema_are_refused_as_belonging_in_secrets_lua() {
    for key in ["token", "schema"] {
        assert_eq!(
            config(&format!("return {{ {key} = 1 }}")).refusal(),
            ConfigError::BelongsInSecrets {
                key: key.into(),
                at: top_of("config.lua"),
            }
        );
    }
}

#[test]
fn a_credential_on_a_remote_is_refused_as_belonging_in_secrets_lua() {
    assert_eq!(
        config("return { remote = { todoist = { url = 'todoist::', api_token = 'x' } } }")
            .refusal(),
        Misread::UnknownRemoteKey {
            key: "api_token".into(),
            at: in_table("remote.todoist", "config.lua"),
            takes: ["url", "stale", "deadline", "path"]
                .map(String::from)
                .to_vec(),
        }
        .into()
    );
}

#[test]
fn a_filter_takes_only_a_query() {
    assert_eq!(
        config("return { filter = { today = { query = 'x', sort = 'due' } } }").refusal(),
        Misread::UnknownKey {
            key: "sort".into(),
            at: in_table("filter.today", "config.lua"),
        }
        .into()
    );
}

#[test]
fn a_map_where_a_list_belongs_and_a_list_where_a_map_belongs_are_refused() {
    assert_eq!(
        config("return { category = { effort = { values = { a = 'light' } } } }").refusal(),
        Misread::WrongType {
            key: "values".into(),
            at: in_table("category.effort", "config.lua"),
            expected: Expected::ListOfTexts,
        }
        .into()
    );
    assert_eq!(
        config("return { remote = { 'todoist::' } }").refusal(),
        Misread::WrongType {
            key: "remote".into(),
            at: top_of("config.lua"),
            expected: Expected::TableOfNamedEntries,
        }
        .into()
    );
}

#[test]
fn a_remote_url_must_name_its_helper() {
    for url in ["::", "a:b::x"] {
        assert_eq!(
            config(&format!(
                "return {{ remote = {{ todoist = {{ url = '{url}' }} }} }}"
            ))
            .refusal(),
            Misread::NotAHelperUrl {
                url: url.into(),
                at: in_table("remote.todoist", "config.lua"),
            }
            .into()
        );
    }
    assert_eq!(
        config("return { remote = { todoist = {} } }").refusal(),
        Misread::Missing {
            key: "url".into(),
            at: in_table("remote.todoist", "config.lua"),
        }
        .into()
    );
}

#[test]
fn a_duration_with_a_sign_is_refused() {
    assert_eq!(
        config("return { remote = { todoist = { url = 'todoist::', stale = '+15m' } } }").refusal(),
        Misread::NotADuration {
            key: "stale".into(),
            text: "+15m".into(),
            at: in_table("remote.todoist", "config.lua"),
        }
        .into()
    );
}
