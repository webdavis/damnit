use super::fake::Fake;
use super::objects::{due, event, task};
use crate::select::{FilterError, Order, Problem, Sort};
use crate::{Completion, Object, Priority};

fn texts(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

fn sorted(words: &[&str], objects: Vec<Object>) -> Vec<String> {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    let sort = Sort::read(&texts(words)).unwrap();
    subjects(sort.arrange(objects, &fake))
}

fn subjects(objects: Vec<Object>) -> Vec<String> {
    objects
        .into_iter()
        .map(|o| o.base().subject.clone())
        .collect()
}

fn named(mut o: Object, subject: &str) -> Object {
    o.base_mut().subject = subject.into();
    o
}

fn due_on(b: u8, day: Option<&str>) -> Object {
    let t = task(b);
    named(
        Object::Task(match day {
            Some(day) => due(t, day.parse().unwrap()),
            None => t,
        }),
        &format!("{b}"),
    )
}

fn with_priority(b: u8, priority: u8) -> Object {
    let mut t = task(b);
    t.priority = Priority::new(priority).unwrap();
    named(Object::Task(t), &format!("{b}"))
}

#[test]
fn a_date_sorts_earliest_first_and_missing_last_both_ways() {
    let objects = || {
        vec![
            due_on(1, None),
            due_on(2, Some("2026-11-22")),
            due_on(3, Some("2026-11-20")),
        ]
    };
    assert_eq!(sorted(&["due"], objects()), ["3", "2", "1"]);
    assert_eq!(sorted(&["due:desc"], objects()), ["2", "3", "1"]);
}

#[test]
fn a_number_sorts_smallest_first() {
    let objects = vec![
        with_priority(1, 3),
        with_priority(2, 1),
        with_priority(3, 2),
    ];
    assert_eq!(sorted(&["priority"], objects), ["2", "3", "1"]);
}

#[test]
fn text_sorts_ignoring_case() {
    let objects = vec![
        named(Object::Task(task(1)), "pear"),
        named(Object::Task(task(2)), "Apple"),
        named(Object::Task(task(3)), "banana"),
    ];
    assert_eq!(sorted(&["subject"], objects), ["Apple", "banana", "pear"]);
}

#[test]
fn a_choice_sorts_by_its_word_and_a_field_the_kind_lacks_counts_as_missing() {
    let objects = || {
        vec![
            named(Object::Task(task(1)), "task"),
            named(Object::Event(event(2, "2026-11-20T09:00[UTC]")), "event"),
        ]
    };
    assert_eq!(sorted(&["kind"], objects()), ["event", "task"]);
    assert_eq!(sorted(&["status:desc"], objects()), ["event", "task"]);
}

#[test]
fn yes_or_no_sorts_false_first() {
    let mut finished = task(1);
    finished.completion = Completion::Done {
        at: Some("2026-11-19T12:00Z".parse().unwrap()),
    };
    let objects = || {
        vec![
            named(Object::Task(finished.clone()), "done"),
            named(Object::Task(task(2)), "open"),
        ]
    };
    assert_eq!(sorted(&["done"], objects()), ["open", "done"]);
    assert_eq!(sorted(&["done:desc"], objects()), ["done", "open"]);
}

#[test]
fn later_keys_break_ties_and_full_ties_keep_the_order_given() {
    let objects = || {
        let mut late = task(1);
        late.due = due(task(9), "2026-11-25".parse().unwrap()).due;
        late.priority = Priority::HIGHEST;
        let late = named(Object::Task(late), "1");
        vec![
            late,
            with_priority(2, 2),
            with_priority(3, 1),
            with_priority(4, 1),
        ]
    };
    assert_eq!(
        sorted(&["priority", "due"], objects()),
        ["1", "3", "4", "2"]
    );
    assert_eq!(sorted(&["priority,due"], objects()), ["1", "3", "4", "2"]);
    assert_eq!(sorted(&[], objects()), ["1", "2", "3", "4"]);
}

#[test]
fn a_mistyped_field_names_the_closest() {
    let error = Sort::read(&texts(&["duee:desc"])).unwrap_err();
    assert_eq!(
        error,
        FilterError {
            field: "duee".into(),
            value: "duee:desc".into(),
            problem: Problem::UnknownField {
                closest: Some("due")
            },
        }
    );
}

#[test]
fn a_direction_other_than_desc_is_a_bad_value() {
    let error = Sort::read(&texts(&["due:down"])).unwrap_err();
    assert_eq!(error.field, "due");
    assert_eq!(error.value, "due:down");
    assert_eq!(error.problem, Problem::BadValue);
}

#[test]
fn offset_and_limit_cut_the_sorted_list() {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    let objects = || (1..=4).map(|b| with_priority(b, 5 - b)).collect::<Vec<_>>();
    let order = |offset, limit| Order {
        sort: Sort::read(&texts(&["priority"])).unwrap(),
        offset,
        limit,
    };
    let arranged = |o: Order| subjects(o.arrange(objects(), &fake));
    assert_eq!(arranged(order(1, Some(2))), ["3", "2"]);
    assert_eq!(arranged(order(3, None)), ["1"]);
    assert_eq!(arranged(order(9, None)), Vec::<String>::new());
    assert_eq!(arranged(order(0, Some(0))), Vec::<String>::new());
}
