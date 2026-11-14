use dam_application::Standing;

use super::support::{secrets, token, tokens, value};

#[test]
fn a_schema_other_than_one_disables_every_token_and_the_rest_still_reads() {
    for schema in ["2", "'one'", "1.5"] {
        let text = format!(
            "return {{ schema = {schema}, remote = {{ todoist = {{ api_token = 'abc' }} }}, \
             token = {{ a = {{ value = {:?} }}, b = {{ value = {:?} }} }} }}",
            value(1),
            value(2)
        );
        let config = secrets(&text).load().unwrap();
        assert_eq!(config.remote("todoist").unwrap().credentials.len(), 1);
        assert_eq!(config.tokens.entries.len(), 2);
        for entry in &config.tokens.entries {
            let Standing::Disabled(why) = &entry.standing else {
                panic!("{entry:?} is usable");
            };
            assert!(why.contains("schema"), "{why}");
        }
    }
}

#[test]
fn a_token_without_a_readable_value_is_left_out_with_a_warning() {
    let (entries, warnings) = tokens(
        "return { token = { a = { via = { 'local' } }, b = { value = 'hunter2' }, c = { value = 7 } } }",
    );
    assert!(entries.is_empty());
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert!(
        warnings.iter().all(|w| !w.contains("hunter2")),
        "{warnings:?}"
    );
}

#[test]
fn a_token_section_that_is_not_tables_never_fails_the_load() {
    for text in ["return { token = 5 }", "return { token = { tool = 5 } }"] {
        let config = secrets(text).load().unwrap();
        assert!(config.tokens.entries.is_empty());
        assert!(!config.tokens.warnings.is_empty(), "{text}");
    }
}

#[test]
fn a_token_that_is_not_a_table_is_ignored_by_name_and_the_others_load() {
    let (entries, warnings) = tokens(&format!(
        "return {{ token = {{ a = 5, b = {{ value = {:?} }} }} }}",
        value(2)
    ));
    let names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names, ["b"]);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("token a"), "{warnings:?}");
}

#[test]
fn a_schema_that_is_not_a_whole_number_says_so() {
    let text = format!(
        "return {{ schema = 1.0, token = {{ a = {{ value = {:?} }} }} }}",
        value(1)
    );
    let (entries, _) = tokens(&text);
    let Standing::Disabled(why) = &entries[0].standing else {
        panic!("{:?} is usable", entries[0]);
    };
    assert_eq!(
        why,
        "schema in secrets.lua must be a whole number, and it is 1.0"
    );
}

#[test]
fn every_token_is_listed_by_name() {
    let config = secrets(&token("")).load().unwrap();
    assert_eq!(config.tokens.names, ["tool"]);
}

#[test]
fn the_config_debug_form_never_holds_a_token_secret() {
    let config = secrets(&token("")).load().unwrap();
    assert!(!format!("{config:?}").contains(&value(1)[21..]));
}

#[test]
fn a_token_with_limits_is_disabled_since_limits_are_not_enforced_yet() {
    let (entries, _) = tokens(&format!(
        "return {{ token = {{ a = {{ value = {:?}, limits = {{ creates_per_day = 50 }} }}, \
         b = {{ value = {:?}, limits = {{}} }} }} }}",
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
        "return {{ token = {{ a = {{ value = {:?} }}, b = {{ value = {:?} }}, c = {{ value = {:?} }} }} }}",
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
