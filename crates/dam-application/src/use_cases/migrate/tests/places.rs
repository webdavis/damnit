use super::*;
use crate::use_cases::migrate::{ConfigPlace, config_places};

#[test]
fn each_category_and_filter_naming_an_old_label_is_listed_with_what_it_would_become() {
    let places = config_places(&configured(), &map(&[("quick", "deep"), ("hard", "light")]));
    assert_eq!(
        places,
        vec![
            ConfigPlace::Category {
                name: "effort".into(),
                values: vec!["deep".into(), "light".into()],
            },
            ConfigPlace::Filter {
                name: "short".into(),
                query: "@deep | due:today".into(),
            },
            ConfigPlace::Filter {
                name: "focus".into(),
                query: "@light & !done".into(),
            },
            ConfigPlace::HandWritten {
                name: "picked".into()
            },
        ]
    );
}

#[test]
fn a_new_label_that_needs_quotes_is_written_quoted() {
    let places = config_places(&configured(), &map(&[("home", "at \"home\"")]));
    assert_eq!(
        places[1],
        ConfigPlace::Filter {
            name: "home".into(),
            query: r#"@"at \"home\"""#.into(),
        }
    );
}

#[test]
fn a_label_no_place_names_lists_nothing() {
    assert!(config_places(&configured(), &map(&[("errand", "chore")])).is_empty());
}
