use jiff::tz::TimeZone;

use super::fake::Fake;
use super::fixture::{TODAY, attached_to, categories, objects};
use super::tables::{one, table};
use crate::select::{Command, Filter, FilterTable, scope};

type Fields<'a> = &'a [(&'a str, &'a str)];

const ALL: [(&str, &str); 2] = [("kind", "any"), ("done", "any")];
const ANY_KIND: [(&str, &str); 1] = [("kind", "any")];

fn build(fields: Fields, any: &[Fields]) -> FilterTable {
    let mut t = table(&fields.iter().map(|(k, v)| (*k, one(v))).collect::<Vec<_>>());
    t.any = any.iter().map(|alt| build(alt, &[])).collect();
    t
}

fn with(defaults: Fields, fields: Fields) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (k, v) in defaults.iter().chain(fields) {
        out.retain(|(key, _)| key != k);
        out.push((k.to_string(), v.to_string()));
    }
    out
}

fn same_objects(old: &str, fields: &[(String, String)], any: &[Fields]) {
    let fake = Fake {
        categories: categories(),
        ..Fake::at("2026-09-18T12:00[UTC]")
    };
    let expr = crate::parse_query(old, &categories()).unwrap();
    let pairs: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let filter = Filter::read(&build(&pairs, any)).unwrap();
    filter.check(&fake.categories).unwrap();
    let scope = scope(&filter, Command::List).unwrap();
    for object in objects() {
        let was = crate::matches(&expr, &object, TODAY, &TimeZone::UTC);
        let now = scope.admits(&object) && filter.matches(&object, &fake);
        assert_eq!(was, now, "{old} on {object:?}");
    }
}

#[test]
fn every_real_query_lists_the_same_objects_with_dates_kept_in_the_persons_zone() {
    let attached = attached_to();
    let cases: Vec<(&str, Vec<(&str, &str)>)> = vec![
        (
            "path:work/ & due:today",
            vec![("path", "work/"), ("due", "today")],
        ),
        ("estimate:max:5m", vec![("estimate", "..5m")]),
        ("estimate:min:1h", vec![("estimate", "1h..")]),
        ("estimate:none", vec![("estimate", "none")]),
        (
            "path:\"kitchen remodel/\"",
            vec![("path", "kitchen remodel/")],
        ),
        ("p1", vec![("priority", "1")]),
        ("p2", vec![("priority", "2")]),
        ("p4", vec![("priority", "4")]),
        ("priority:1", vec![("priority", "1")]),
        ("completed:today", vec![("completed_at", "today")]),
        ("completed:yesterday", vec![("completed_at", "today-1")]),
        ("completed:tomorrow", vec![("completed_at", "tomorrow")]),
        ("completed:none", vec![("completed_at", "none")]),
        ("kind:task", vec![("kind", "task")]),
        ("kind:event", vec![("kind", "event")]),
        (
            "effort:hard & !done",
            vec![("category", "effort=hard"), ("done", "false")],
        ),
        (
            "@light & !done",
            vec![("labels", "light"), ("done", "false")],
        ),
        (
            "@quick & !done",
            vec![("labels", "quick"), ("done", "false")],
        ),
        ("@cli & p1", vec![("labels", "cli"), ("priority", "1")]),
        ("@x & due:today", vec![("labels", "x"), ("due", "today")]),
        ("due:today", vec![("due", "today")]),
        ("due:tomorrow", vec![("due", "tomorrow")]),
        ("due:2026-09-19", vec![("due", "2026-09-19")]),
        ("due:this-week", vec![("due", "week")]),
        ("due:next-week", vec![("due", "week+1")]),
        ("due:none", vec![("due", "none")]),
        ("deadline:none", vec![("deadline", "none")]),
        ("focus:today", vec![("focus.day", "today")]),
        ("overdue", vec![("overdue", "true")]),
        (" overdue", vec![("overdue", "true")]),
        (
            "due:today & overdue",
            vec![("due", "today"), ("overdue", "true")],
        ),
        ("!done", vec![("done", "false")]),
        (" !done ", vec![("done", "false")]),
        ("subject:oat", vec![("subject", "oat")]),
        ("subject:OAT", vec![("subject", "OAT")]),
        (r#"subject:"\"best\"""#, vec![("subject", "\"best\"")]),
        (r#"subject:"\\""#, vec![("subject", "\\")]),
        ("start:today", vec![("start", "today")]),
        ("transparency:free", vec![("transparency", "free")]),
        ("path:\"other place/\"", vec![("path", "other place/")]),
        (
            "kind:task & path:work/ & !done",
            vec![("kind", "task"), ("path", "work/"), ("done", "false")],
        ),
    ];
    let mut cases = cases;
    let attached_query = format!("attached:{attached}");
    cases.push((&attached_query, vec![("attached", &attached)]));
    for path in [
        "home/",
        "clients/",
        "work/",
        "garden/",
        "errands",
        "damnit/cli/",
        "damnit/",
        "cli/",
        "other/",
        "work",
        "inbox",
    ] {
        same_objects(&format!("path:{path}"), &with(&ALL, &[("path", path)]), &[]);
    }
    for label in [
        "home", "quick", "deep", "app", "template", "at", "cli", "work",
    ] {
        let fields = with(&ALL, &[("labels", label)]);
        same_objects(&format!("@{label}"), &fields, &[]);
        same_objects(&format!("label:{label}"), &fields, &[]);
    }
    for (old, fields) in cases {
        let defaults: Fields = if old.contains("done") {
            &ANY_KIND
        } else {
            &ALL
        };
        same_objects(old, &with(defaults, &fields), &[]);
    }
}

#[test]
fn every_real_or_query_lists_the_same_objects_through_any() {
    let today_or_late: [Fields; 2] = [&[("due", "today")], &[("overdue", "true")]];
    same_objects("due:today | overdue", &with(&ALL, &[]), &today_or_late);
    let open = with(&ANY_KIND, &[("done", "false")]);
    same_objects("(due:today | overdue) & !done", &open, &today_or_late);
    same_objects("!done & (due:today | overdue)", &open, &today_or_late);
    for label in ["quick", "deep"] {
        let alternatives: [Fields; 2] = [&[("labels", label)], &[("due", "today")]];
        same_objects(
            &format!("@{label} | due:today"),
            &with(&ALL, &[]),
            &alternatives,
        );
    }
    for root in ["", "damnit/"] {
        let (cli, docs) = (format!("{root}cli/"), format!("not:{root}docs/"));
        let alternatives: [Fields; 3] = [
            &[("path", &cli)],
            &[("path", &docs)],
            &[("labels", "not:x")],
        ];
        let old = format!("path:{root}cli/ | !(path:{root}docs/ & @x)");
        same_objects(&old, &with(&ALL, &[]), &alternatives);
    }
}

#[test]
fn before_and_after_are_exclusive_so_the_span_shifts_a_day() {
    let before = with(&ALL, &[("due", "..2026-09-17")]);
    same_objects("due:before:2026-09-18", &before, &[]);
    let after = with(&ALL, &[("due", "2026-09-19..")]);
    same_objects("due:after:2026-09-18", &after, &[]);
}
