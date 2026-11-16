use super::category::*;
use crate::error::CliError;
use dam_domain::{Categories, Category};

fn categories() -> Categories {
    Categories::new(vec![
        Category {
            name: "effort".into(),
            values: vec!["light".into(), "deep".into()],
            exclusive: true,
        },
        Category {
            name: "area".into(),
            values: vec!["ui".into(), "db".into()],
            exclusive: false,
        },
    ])
    .unwrap()
}

fn asked(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(g, l)| (g.to_string(), l.to_string()))
        .collect()
}

fn refusal(pairs: &[(&str, &str)], categories: &Categories) -> String {
    match category_labels(categories, &asked(pairs)) {
        Err(CliError::Usage(text)) => text,
        other => panic!("not refused: {other:?}"),
    }
}

#[test]
fn a_label_of_the_group_is_added() {
    let got =
        category_labels(&categories(), &asked(&[("effort", "deep"), ("area", "ui")])).unwrap();
    assert_eq!(got.add, vec!["deep", "ui"]);
    assert!(got.remove.is_empty());
}

#[test]
fn none_takes_off_every_label_of_the_group() {
    let got = category_labels(&categories(), &asked(&[("effort", "none")])).unwrap();
    assert!(got.add.is_empty());
    assert_eq!(got.remove, vec!["light", "deep"]);
}

#[test]
fn an_unknown_group_is_refused_listing_the_groups() {
    let text = refusal(&[("size", "big")], &categories());
    assert!(text.contains("--category size=big"), "{text}");
    assert!(text.contains("effort, area"), "{text}");
    assert!(text.contains("dam category list"), "{text}");
}

#[test]
fn a_label_outside_the_group_is_refused_listing_its_labels() {
    let text = refusal(&[("effort", "huge")], &categories());
    assert!(text.contains("light, deep"), "{text}");
    assert!(text.contains("--category effort=light"), "{text}");
}

#[test]
fn with_no_groups_declared_the_refusal_names_the_command_that_declares_one() {
    let text = refusal(&[("effort", "deep")], &Categories::default());
    assert!(text.contains("dam category create"), "{text}");
}
