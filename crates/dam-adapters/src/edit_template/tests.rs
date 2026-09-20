
use dam_application::EditFields;
use dam_domain::{Object, Oid, Priority, Task, When};
use jiff::civil::date;

fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

fn task() -> Object {
    let mut t = Task::new(oid(1), "milk");
    t.priority = Priority::new(2).unwrap();
    t.base.labels.insert("errand".into());
    Object::Task(t)
}

fn tz() -> jiff::tz::TimeZone {
    jiff::tz::TimeZone::UTC
}

#[test]
fn an_unchanged_template_parses_to_no_edits() {
    let text = super::render_template(&task());
    let fields = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap();
    assert_eq!(fields, EditFields::default());
}

#[test]
fn changed_values_become_edits_and_labels_diff() {
    let text = super::render_template(&task())
        .replace("subject = \"milk\"", "subject = \"oat milk\"")
        .replace("due = \"\"", "due = \"tomorrow\"")
        .replace("labels = [\"errand\"]", "labels = [\"grocery\"]");
    let fields = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap();
    assert_eq!(fields.subject.as_deref(), Some("oat milk"));
    assert_eq!(fields.due, Some(Some(When::Day(date(2026, 9, 19)))));
    assert_eq!(fields.add_labels, vec!["grocery".to_string()]);
    assert_eq!(fields.remove_labels, vec!["errand".to_string()]);
}

#[test]
fn a_bad_line_is_named() {
    let text = super::render_template(&task()).replace("priority = 2", "priority = \"high\"");
    let err = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap_err();
    assert!(err.starts_with("line 5:"), "{err}");
    assert!(err.contains("priority"));
}

fn meeting() -> Object {
    let mut e = dam_domain::Event::new(
        oid(2),
        "standup",
        When::Day(date(2026, 9, 18)),
        When::Day(date(2026, 9, 18)),
    );
    e.location = Some("room 2".into());
    Object::Event(e)
}

#[test]
fn an_unchanged_event_template_parses_to_no_edits() {
    let text = super::render_template(&meeting());
    let fields = super::parse_template(&text, &meeting(), date(2026, 9, 18), &tz()).unwrap();
    assert_eq!(fields, EditFields::default());
}

#[test]
fn hostile_subject_text_survives_a_render_and_a_parse_unchanged() {
    for hostile in [
        "a \"quoted\" subject",
        "two\nlines",
        "a back\\slash",
        "a \u{7}bell and a \u{0}nul",
        "a \u{200b}zero width space and a \u{202e}override",
        "emoji \u{1f600} and an accent caf\u{e9}",
    ] {
        let mut t = Task::new(oid(1), hostile);
        t.priority = Priority::new(2).unwrap();
        let object = Object::Task(t);
        let text = super::render_template(&object);
        let fields = super::parse_template(&text, &object, date(2026, 9, 18), &tz())
            .unwrap_or_else(|e| panic!("{hostile:?} did not round trip: {e}"));
        assert_eq!(fields, EditFields::default(), "{hostile:?}");
    }
}

#[test]
fn a_misspelled_field_is_refused_rather_than_dropped() {
    let text = super::render_template(&task()).replace("subject =", "subjectt =");
    let err = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap_err();
    assert!(err.contains("subjectt"), "{err}");
    assert!(err.starts_with("line 3:"), "{err}");
}

#[test]
fn a_field_belonging_to_the_other_kind_is_refused_by_name() {
    let text = format!(
        "{}start = \"2026-09-19\"\n",
        super::render_template(&task())
    );
    let err = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap_err();
    assert!(err.contains("start"), "{err}");
}

#[test]
fn broken_toml_names_its_line() {
    let text = super::render_template(&task()).replace("body = \"\"", "body = \"unterminated");
    let err = super::parse_template(&text, &task(), date(2026, 9, 18), &tz()).unwrap_err();
    assert!(err.starts_with("line 4:"), "{err}");
}
