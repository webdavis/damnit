use dam_application::{Expected, FilterRule, Misread};

use super::support::{config, in_table};

fn rule_of(filter: &str) -> FilterRule {
    let text = format!("return {{ filter = {{ slipping = {filter} }} }}");
    config(&text)
        .load()
        .unwrap()
        .filter("slipping")
        .unwrap()
        .rule
        .clone()
}

#[test]
fn a_filter_with_match_alone_runs_its_function() {
    assert_eq!(
        rule_of("{ match = function(task, dam) return true end }"),
        FilterRule::Function
    );
}

#[test]
fn a_filter_with_query_and_match_narrows_by_the_query_first() {
    assert_eq!(
        rule_of("{ query = 'path:work/', match = function() return true end }"),
        FilterRule::QueryThenFunction("path:work/".into())
    );
}

#[test]
fn a_filter_with_neither_query_nor_match_is_refused() {
    assert_eq!(
        config("return { filter = { slipping = {} } }").refusal(),
        Misread::Missing {
            key: "query or match".into(),
            at: in_table("filter.slipping", "config.lua"),
        }
        .into()
    );
}

#[test]
fn a_match_that_is_not_a_function_is_refused_by_name() {
    assert_eq!(
        config("return { filter = { slipping = { match = 'due:today' } } }").refusal(),
        Misread::WrongType {
            key: "match".into(),
            at: in_table("filter.slipping", "config.lua"),
            expected: Expected::Function,
        }
        .into()
    );
}
