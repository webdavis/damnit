use super::fake::Fake;
use super::objects::{due, labelled, task};
use super::tables::{either, one, table};
use crate::select::{Filter, FilterTable};
use crate::{Categories, Category, Object, Path};

fn hit(t: &FilterTable, object: &Object) -> bool {
    let fake = Fake::at("2026-11-20T12:00[UTC]");
    Filter::read(t).unwrap().matches(object, &fake)
}

fn at_work(mut o: Object) -> Object {
    o.base_mut().path = Path::parse("work/").unwrap();
    o
}

#[test]
fn at_least_one_any_alternative_must_hold_as_well_as_every_other_condition() {
    let filter = either(
        table(&[("path", one("work/"))]),
        vec![
            table(&[("labels", one("urgent"))]),
            table(&[("due", one("..today"))]),
        ],
    );
    let urgent = at_work(labelled(Object::Task(task(1)), &["urgent"]));
    let late = at_work(Object::Task(due(task(2), "2026-11-19".parse().unwrap())));
    let neither = at_work(Object::Task(task(3)));
    let urgent_at_home = labelled(Object::Task(task(4)), &["urgent"]);
    assert!(hit(&filter, &urgent) && hit(&filter, &late));
    assert!(!hit(&filter, &neither) && !hit(&filter, &urgent_at_home));
}

#[test]
fn any_groups_nest() {
    let filter = either(
        table(&[]),
        vec![
            table(&[("labels", one("a"))]),
            either(
                table(&[("labels", one("b"))]),
                vec![
                    table(&[("labels", one("c"))]),
                    table(&[("labels", one("d"))]),
                ],
            ),
        ],
    );
    let has = |labels: &[&str]| hit(&filter, &labelled(Object::Task(task(1)), labels));
    assert!(has(&["a"]) && has(&["b", "c"]) && has(&["b", "d"]));
    assert!(!has(&["b"]) && !has(&["c"]));
}

#[test]
fn a_category_check_reaches_inside_any() {
    let categories = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["deep".into()],
        exclusive: false,
    }])
    .unwrap();
    let t = either(table(&[]), vec![table(&[("category", one("effort=home"))])]);
    assert!(Filter::read(&t).unwrap().check(&categories).is_err());
    let ok = either(table(&[]), vec![table(&[("category", one("effort=deep"))])]);
    assert!(Filter::read(&ok).unwrap().check(&categories).is_ok());
}
