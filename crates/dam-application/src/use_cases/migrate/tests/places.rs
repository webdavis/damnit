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
                table: short("deep"),
            },
            ConfigPlace::Filter {
                name: "focus".into(),
                table: table(&[("category", "effort=light")], vec![]),
            },
            ConfigPlace::HandWritten {
                name: "picked".into()
            },
        ]
    );
}

#[test]
fn a_new_label_holding_a_comma_is_written_exact() {
    let places = config_places(&configured(), &map(&[("home", "at,home")]));
    assert_eq!(
        places[1],
        ConfigPlace::Filter {
            name: "home".into(),
            table: table(&[("labels", "=at,home")], vec![]),
        }
    );
}

#[test]
fn a_label_no_place_names_lists_nothing() {
    assert!(config_places(&configured(), &map(&[("errand", "chore")])).is_empty());
}
