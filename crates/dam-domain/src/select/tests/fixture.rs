use jiff::ToSpan;

use super::objects::oid;
use crate::{
    Categories, Category, Completion, Date, Event, Focus, Object, Path, Priority, Task,
    Transparency, When,
};

pub const TODAY: Date = jiff::civil::date(2026, 9, 18);

const PATHS: [&str; 15] = [
    "work/",
    "work/dam/",
    "home/",
    "clients/acme/",
    "garden/",
    "errands/",
    "damnit/cli/",
    "damnit/docs/",
    "cli/",
    "docs/",
    "kitchen remodel/tiles/",
    "other place/",
    "inbox/",
    "other/",
    "",
];
const LABELS: [&str; 11] = [
    "quick", "deep", "light", "hard", "home", "cli", "app", "template", "at", "work", "x",
];
const DUE_DAYS: [Option<i32>; 9] = [
    Some(0),
    Some(-1),
    Some(1),
    Some(-4),
    Some(2),
    Some(3),
    Some(5),
    Some(9),
    None,
];
const SUBJECTS: [&str; 4] = ["Buy oat milk", "get \"best\" quotes", "a \\ b", "plain"];

pub fn categories() -> Categories {
    let effort = Category {
        name: "effort".into(),
        values: vec!["light".into(), "deep".into(), "hard".into()],
        exclusive: false,
    };
    Categories::new(vec![effort]).unwrap()
}

fn day(offset: i32) -> Date {
    TODAY.checked_add(offset.days()).unwrap()
}

fn base(o: &mut Object, i: usize) {
    let b = o.base_mut();
    b.path = Path::parse(PATHS[i % PATHS.len()]).unwrap();
    b.labels = [LABELS[i % LABELS.len()], LABELS[(i * 3 + 1) % LABELS.len()]]
        .into_iter()
        .filter(|_| i % 6 != 5)
        .map(String::from)
        .collect();
    b.subject = SUBJECTS[i % SUBJECTS.len()].into();
}

fn task(i: usize) -> Object {
    let mut t = Task::new(oid(i as u8), "");
    t.due = DUE_DAYS[i % DUE_DAYS.len()].map(|d| When::Day(day(d)));
    t.deadline = i.is_multiple_of(4).then(|| day(i as i32 % 3));
    t.priority = Priority::new((i % 4) as u8 + 1).unwrap();
    t.estimate = [None, Some(5), Some(60), Some(90), Some(3)][i % 5];
    t.event = i.is_multiple_of(10).then(|| oid(200));
    t.focus = (i % 5 == 1).then(|| Focus {
        day: day(i as i32 % 2),
        set_by: "me".into(),
        set_at: "2026-09-18T08:00Z".parse().unwrap(),
    });
    if i.is_multiple_of(3) {
        let at = (i % 4 != 3).then(|| {
            day((i / 3 % 3) as i32 - 1)
                .to_zoned(jiff::tz::TimeZone::UTC)
                .unwrap()
                .timestamp()
                + 12.hours()
        });
        t.completion = Completion::Done { at };
    }
    let mut o = Object::Task(t);
    base(&mut o, i);
    o
}

fn event(i: usize) -> Object {
    let start = When::Day(day(i as i32 % 9 - 3));
    let mut e = Event::new(oid(100 + i as u8), "", start.clone(), start);
    e.transparency = [Transparency::Busy, Transparency::Free][i % 2];
    let mut o = Object::Event(e);
    base(&mut o, i + 2);
    o
}

fn docs_labelled_x() -> Object {
    let mut o = task(1);
    o.base_mut().path = Path::parse("damnit/docs/").unwrap();
    o.base_mut().labels = ["x".to_string()].into();
    o
}

pub fn objects() -> Vec<Object> {
    (0..90)
        .map(task)
        .chain((0..30).map(event))
        .chain([docs_labelled_x()])
        .collect()
}

pub fn attached_to() -> String {
    oid(200).to_string()
}
