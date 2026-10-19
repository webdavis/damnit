mod build;
mod wire;

use super::*;
use crate::api::{Due, Item, SyncResponse};
use build::{item, project, section, sync_of};

fn sync() -> SyncResponse {
    SyncResponse {
        projects: vec![
            project("p1", "Work"),
            project("p2", "Client").under("p1").archived(),
        ],
        sections: vec![section("s1", "Now").under("p1")],
        items: vec![
            Item {
                description: "2%".into(),
                priority: 4,
                due: Some(Due {
                    date: "2026-09-25".into(),
                    is_recurring: true,
                    string: "every week".into(),
                }),
                labels: vec!["errand".into()],
                ..item("i1", "milk").under("p1").in_section("s1")
            },
            Item {
                checked: true,
                ..item("i2", "oat")
                    .under("p1")
                    .in_section("s1")
                    .child_of("i1")
            },
            Item {
                priority: 2,
                ..item("i3", "loose").under("p1")
            },
        ],
    }
}

#[test]
fn paths_follow_project_section_and_parent_names() {
    let t = Tree::from_sync(&sync());
    assert_eq!(t.project_path("p1").as_deref(), Some("Work/"));
    assert_eq!(t.project_path("p2").as_deref(), Some("Work/Client/"));
    assert_eq!(t.section_path("s1").as_deref(), Some("Work/Now/"));
    let items = &sync().items;
    assert_eq!(t.item_path(&items[0]).as_deref(), Some("Work/Now/"));
    assert_eq!(t.item_path(&items[1]).as_deref(), Some("Work/Now/milk/"));
    assert_eq!(t.item_path(&items[2]).as_deref(), Some("Work/"));
}

#[test]
fn reverse_lookups_find_ids_by_path() {
    let t = Tree::from_sync(&sync());
    assert_eq!(t.project_id_for("Work/"), Some("p1"));
    assert_eq!(t.project_id_for("Work/Client/"), Some("p2"));
    assert_eq!(t.section_id_for("Work/Now/"), Some("s1"));
    assert_eq!(t.item_id_for("Work/Now/", "milk"), Some("i1"));
    assert!(t.project_id_for("Nope/").is_none());
}

#[test]
fn a_slash_in_a_name_becomes_one_dam_segment_not_two() {
    let s = sync_of(vec![project("p1", "a/b")], vec![]);
    let t = Tree::from_sync(&s);
    assert_eq!(t.project_path("p1").as_deref(), Some("a-b/"));
}

#[test]
fn a_project_parent_cycle_returns_none_instead_of_overflowing() {
    let s = sync_of(
        vec![
            project("p1", "A").under("p2"),
            project("p2", "B").under("p1"),
        ],
        vec![],
    );
    let t = Tree::from_sync(&s);
    assert!(t.project_path("p1").is_none());
    assert!(t.project_path("p2").is_none());
}

#[test]
fn an_item_parent_cycle_returns_none_instead_of_overflowing() {
    let s = sync_of(
        vec![project("p1", "Work")],
        vec![
            item("i1", "one").under("p1").child_of("i2"),
            item("i2", "two").under("p1").child_of("i1"),
        ],
    );
    let t = Tree::from_sync(&s);
    assert!(t.item_path(&s.items[0]).is_none());
    assert!(item_to_wire(&t, &s.items[0]).is_none());
}

#[test]
fn a_parent_content_with_a_slash_stays_one_segment_of_the_child_path() {
    let s = sync_of(
        vec![project("p1", "Work")],
        vec![
            item("i1", "buy milk/oat").under("p1"),
            item("i2", "at the shop").under("p1").child_of("i1"),
        ],
    );
    let t = Tree::from_sync(&s);
    let child = &s.items[1];
    assert_eq!(t.item_path(child).as_deref(), Some("Work/buy milk-oat/"));
    assert_eq!(item_to_wire(&t, child).unwrap().path, "Work/buy milk-oat/");
}

#[test]
fn a_blank_parent_content_names_the_segment_after_its_id() {
    let s = sync_of(
        vec![project("p1", "Work")],
        vec![
            item("i1", "   ").under("p1"),
            item("i2", "child").under("p1").child_of("i1"),
        ],
    );
    let t = Tree::from_sync(&s);
    let child = &s.items[1];
    assert_eq!(t.item_path(child).as_deref(), Some("Work/untitled-i1/"));
    assert_eq!(item_to_wire(&t, child).unwrap().path, "Work/untitled-i1/");
}
