use super::*;
use crate::{Categories, Category};

fn effort() -> Categories {
    Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["quick".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap()
}

fn found(text: &str) -> Vec<(String, &str)> {
    label_terms(text, &effort())
        .unwrap()
        .into_iter()
        .map(|term| (term.label, &text[term.at]))
        .collect()
}

#[test]
fn each_label_term_is_found_with_the_text_it_was_written_as() {
    assert_eq!(
        found(r#"(@quick | effort:deep) & !label:"two words" & due:today & subject:quick"#),
        vec![
            ("quick".to_string(), "@quick"),
            ("deep".to_string(), "effort:deep"),
            ("two words".to_string(), r#"label:"two words""#),
        ]
    );
}

#[test]
fn a_query_that_does_not_parse_finds_no_terms() {
    assert_eq!(
        label_terms("effort:nonsense", &effort()),
        Err(QueryError::NotAValue {
            category: "effort".into(),
            value: "nonsense".into()
        })
    );
}
