use dam_domain::{Object, Oid, Task, When};
use jiff::Zoned;
use jiff::civil::date;

fn task() -> Object {
    Object::Task(Task::new(
        Oid::generate(&mut |x: &mut [u8]| x.fill(1)),
        "milk",
    ))
}

fn now() -> Zoned {
    "2026-09-18T09:00[UTC]".parse().unwrap()
}

fn edited(key: &str, value: &str) -> Result<dam_application::EditFields, String> {
    let text = super::render_template(&task())
        .replace(&format!("{key} = \"\""), &format!("{key} = \"{value}\""));
    super::parse_template(&text, &task(), date(2026, 9, 18), &now())
}

#[test]
fn the_editor_reads_a_date_as_the_flags_read_it() {
    assert_eq!(
        edited("due", "today+2").unwrap().due,
        Some(Some(When::Day(date(2026, 9, 20))))
    );
    let later: Zoned = "2026-09-18T10:30[UTC]".parse().unwrap();
    assert_eq!(
        edited("due", "now+90m").unwrap().due,
        Some(Some(When::At(later)))
    );
    assert_eq!(
        edited("deadline", "today+3").unwrap().deadline,
        Some(Some(date(2026, 9, 21)))
    );
}

#[test]
fn the_editor_refuses_a_date_that_names_more_than_one_day() {
    for key in ["due", "deadline"] {
        let err = edited(key, "2026-11").unwrap_err();
        assert!(err.contains("one day or instant"), "{err}");
        assert!(err.contains("today+3"), "{err}");
    }
}
