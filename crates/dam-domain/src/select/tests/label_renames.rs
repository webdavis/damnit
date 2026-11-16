use super::tables::{either, many, one, table};

fn renamed(label: &str) -> Option<String> {
    match label {
        "quick" => Some("light".into()),
        "hard" => Some("deep".into()),
        "home" => Some("at,home".into()),
        _ => None,
    }
}

#[test]
fn labels_are_renamed_inside_lists_not_and_any() {
    let t = either(
        table(&[
            ("labels", many(&["quick", "not:hard,x"])),
            ("subject", one("quick")),
        ]),
        vec![table(&[("labels", one("=quick"))])],
    );
    let moved = either(
        table(&[
            ("labels", many(&["light", "not:deep,x"])),
            ("subject", one("quick")),
        ]),
        vec![table(&[("labels", one("=light"))])],
    );
    assert_eq!(t.labels_renamed(&renamed), moved);
}

#[test]
fn a_category_value_names_its_label_after_the_group() {
    let t = table(&[(
        "category",
        many(&["effort=quick,hard", "not:effort=hard", "effort"]),
    )]);
    let moved = table(&[(
        "category",
        many(&["effort=light,deep", "not:effort=deep", "effort"]),
    )]);
    assert_eq!(t.labels_renamed(&renamed), moved);
}

#[test]
fn a_new_label_holding_a_comma_is_written_exact() {
    let t = table(&[("labels", one("home"))]);
    assert_eq!(
        t.labels_renamed(&renamed),
        table(&[("labels", one("=at,home"))])
    );
}
