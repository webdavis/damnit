use crate::select::chain_to_itself;

fn reaches(name: &str) -> Vec<String> {
    let names: &[&str] = match name {
        "a" => &["b", "c"],
        "b" => &["c"],
        "c" => &["a"],
        "d" => &["d"],
        "e" => &["c", "f"],
        _ => &[],
    };
    names.iter().map(|n| n.to_string()).collect()
}

#[test]
fn a_filter_that_reaches_itself_names_the_chain() {
    assert_eq!(
        chain_to_itself("a", reaches),
        Some(vec!["a".into(), "b".into(), "c".into(), "a".into()])
    );
    assert_eq!(
        chain_to_itself("d", reaches),
        Some(vec!["d".into(), "d".into()])
    );
}

#[test]
fn a_filter_that_reaches_a_loop_it_is_not_in_is_kept() {
    assert_eq!(chain_to_itself("e", reaches), None);
    assert_eq!(chain_to_itself("f", reaches), None);
}
