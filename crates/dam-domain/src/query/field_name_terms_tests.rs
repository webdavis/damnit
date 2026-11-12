use super::*;
use crate::Categories;

fn same(key: &str, alias: &str, value: &str) {
    let categories = Categories::default();
    assert_eq!(
        parse(&format!("{key}:{value}"), &categories).unwrap(),
        parse(&format!("{alias}:{value}"), &categories).unwrap(),
        "{key}: and {alias}: read differently"
    );
}

#[test]
fn a_query_term_named_after_a_json_key_reads_as_its_short_form() {
    same("labels", "label", "errand");
    same("completed_at", "completed", "today");
}
