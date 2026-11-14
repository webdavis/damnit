use dam_application::{Expected, Misread, Standing, TokenFault, TokenWarning};

use super::support::{in_table, secrets, token, tokens, value};

#[test]
fn a_schema_other_than_one_disables_every_token_and_the_rest_still_reads() {
    for (schema, fault) in [
        ("2", TokenFault::NewerSchema { found: 2, reads: 1 }),
        (
            "'one'",
            TokenFault::SchemaNotWhole {
                found: "\"one\"".into(),
            },
        ),
        (
            "1.5",
            TokenFault::SchemaNotWhole {
                found: "1.5".into(),
            },
        ),
    ] {
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
            assert_eq!(entry.standing, Standing::Disabled(fault.clone()));
        }
    }
}

#[test]
fn a_token_without_a_readable_value_is_left_out_with_a_warning() {
    let (entries, warnings) = tokens(
        "return { token = { a = { via = { 'local' } }, b = { value = 'hunter2' }, c = { value = 7 } } }",
    );
    assert!(entries.is_empty());
    let ignored = |name: &str, why| TokenWarning::Ignored {
        name: name.into(),
        why,
    };
    assert_eq!(
        warnings,
        [
            ignored("a", TokenFault::NoValue),
            ignored("b", TokenFault::ValueNotAToken),
            ignored("c", TokenFault::ValueNotText),
        ]
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
    assert_eq!(
        warnings,
        [TokenWarning::Ignored {
            name: "a".into(),
            why: Misread::WrongType {
                key: "a".into(),
                at: in_table("token", "secrets.lua"),
                expected: Expected::Table,
            }
            .into(),
        }]
    );
}

#[test]
fn a_schema_that_is_not_a_whole_number_says_so() {
    let text = format!(
        "return {{ schema = 1.0, token = {{ a = {{ value = {:?} }} }} }}",
        value(1)
    );
    let (entries, _) = tokens(&text);
    assert_eq!(
        entries[0].standing,
        Standing::Disabled(TokenFault::SchemaNotWhole {
            found: "1.0".into(),
        })
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
    assert_eq!(
        entries[0].standing,
        Standing::Disabled(TokenFault::LimitsNotEnforced)
    );
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
        assert_eq!(
            standing(name),
            &Standing::Disabled(TokenFault::SharesValue {
                with: vec![other.to_string()],
            })
        );
    }
    assert!(matches!(standing("b"), Standing::Usable(_)));
}
