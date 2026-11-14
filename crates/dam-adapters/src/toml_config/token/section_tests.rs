use dam_application::Standing;

use super::tests::{tokens, value};
use crate::toml_config::parse_config;

#[test]
fn a_newer_schema_disables_every_token_and_the_rest_still_reads() {
    let text = format!(
        "schema = 2\n[remote.todoist]\nurl = \"todoist::\"\n[token.a]\nvalue = \"{}\"\n[token.b]\nvalue = \"{}\"\n",
        value(1),
        value(2)
    );
    let config = parse_config(&text).unwrap();
    assert!(config.remote("todoist").is_some());
    assert_eq!(config.tokens.entries.len(), 2);
    for entry in &config.tokens.entries {
        let Standing::Disabled(why) = &entry.standing else {
            panic!("{entry:?} is usable");
        };
        assert!(why.contains("schema 2"), "{why}");
    }
}

#[test]
fn a_token_without_a_readable_value_is_left_out_with_a_warning() {
    let (entries, warnings) = tokens(
        "[token.a]\nvia = [\"local\"]\n[token.b]\nvalue = \"hunter2\"\n[token.c]\nvalue = 7\n",
    );
    assert!(entries.is_empty());
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert!(
        warnings.iter().all(|w| !w.contains("hunter2")),
        "{warnings:?}"
    );
}

#[test]
fn a_token_section_that_is_not_tables_never_fails_the_config() {
    for text in ["token = 5", "[token]\ntool = 5", "schema = \"one\""] {
        let config = parse_config(text).unwrap();
        assert!(config.tokens.entries.is_empty());
    }
    assert!(
        !parse_config("token = 5")
            .unwrap()
            .tokens
            .warnings
            .is_empty()
    );
}

#[test]
fn the_config_debug_form_never_holds_a_token_secret() {
    let config = parse_config(&format!("[token.tool]\nvalue = \"{}\"\n", value(1))).unwrap();
    let secret = &value(1)[21..];
    assert!(!format!("{config:?}").contains(secret));
}

#[test]
fn a_token_with_limits_is_disabled_since_limits_are_not_enforced_yet() {
    let (entries, _) = tokens(&format!(
        "[token.a]\nvalue = \"{}\"\nlimits = {{ creates_per_day = 50 }}\n[token.b]\nvalue = \"{}\"\nlimits = {{}}\n",
        value(1),
        value(2)
    ));
    let Standing::Disabled(why) = &entries[0].standing else {
        panic!("{:?} is usable", entries[0]);
    };
    assert!(why.contains("limits are not enforced yet"), "{why}");
    assert!(matches!(entries[1].standing, Standing::Usable(_)));
}

#[test]
fn tokens_sharing_one_value_are_all_disabled_by_name_and_the_value_is_never_named() {
    let (entries, _) = tokens(&format!(
        "[token.a]\nvalue = \"{}\"\n[token.b]\nvalue = \"{}\"\n[token.c]\nvalue = \"{}\"\n",
        value(1),
        value(2),
        value(1)
    ));
    let standing = |name: &str| &entries.iter().find(|e| e.name == name).unwrap().standing;
    for (name, other) in [("a", "c"), ("c", "a")] {
        let Standing::Disabled(why) = standing(name) else {
            panic!("token {name} is usable");
        };
        assert!(why.contains(&format!("token {other}")), "{why}");
        assert!(!why.contains(&value(1)[8..]), "{why}");
    }
    assert!(matches!(standing("b"), Standing::Usable(_)));
}
