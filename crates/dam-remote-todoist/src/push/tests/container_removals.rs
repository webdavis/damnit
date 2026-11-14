use dam_protocol::Mutation;

use super::fixtures::{mutation, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn project(id: &str, parent_id: Option<&str>) -> serde_json::Value {
    serde_json::json!({"id": id, "name": id, "parent_id": parent_id, "is_deleted": false, "is_archived": false, "inbox_project": false})
}

fn section(id: &str, project_id: &str) -> serde_json::Value {
    serde_json::json!({"id": id, "name": id, "project_id": project_id, "is_deleted": false})
}

fn item(id: &str, section_id: Option<&str>, parent_id: Option<&str>) -> serde_json::Value {
    serde_json::json!({"id": id, "content": id, "project_id": "p1", "section_id": section_id, "parent_id": parent_id, "priority": 1, "checked": false, "is_deleted": false})
}

fn upstream(
    projects: Vec<serde_json::Value>,
    sections: Vec<serde_json::Value>,
    items: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({"sync_token": "t", "full_sync": true, "projects": projects, "sections": sections, "items": items})
}

fn removal(n: u8, remote_id: &str) -> Mutation {
    mutation("delete", n, Some(remote_id), None)
}

fn refused_without_a_command(held: serde_json::Value, remote_id: &str, holds: &str) {
    let api = FakeTodoist::new(held);
    let response = push(&api, vec![removal(1, remote_id)]).unwrap();
    let why = response.results[0].why().unwrap_or_default().to_string();
    assert!(!response.results[0].ok(), "{remote_id} was removed");
    assert!(why.contains(holds), "{why}");
    assert!(why.contains("pull, then push"), "{why}");
    assert!(api.recorded.borrow().executed.is_empty());
}

#[test]
fn a_task_todoist_shows_holding_a_subtask_is_not_removed() {
    let held = upstream(
        vec![project("p1", None)],
        vec![],
        vec![item("t1", None, None), item("t2", None, Some("t1"))],
    );
    refused_without_a_command(held, "i:t1", "1 task");
}

#[test]
fn a_section_todoist_shows_holding_a_task_is_not_removed() {
    let held = upstream(
        vec![project("p1", None)],
        vec![section("s1", "p1")],
        vec![item("t1", Some("s1"), None)],
    );
    refused_without_a_command(held, "s:s1", "1 task");
}

#[test]
fn a_project_todoist_shows_holding_a_task_a_section_or_a_project_is_not_removed() {
    let holding_a_task = upstream(
        vec![project("p1", None)],
        vec![],
        vec![item("t1", None, None)],
    );
    refused_without_a_command(holding_a_task, "p:p1", "1 task");
    let holding_a_section = upstream(vec![project("p1", None)], vec![section("s1", "p1")], vec![]);
    refused_without_a_command(holding_a_section, "p:p1", "1 section");
    let holding_a_project = upstream(
        vec![project("p1", None), project("p2", Some("p1"))],
        vec![],
        vec![],
    );
    refused_without_a_command(holding_a_project, "p:p1", "1 project");
}

#[test]
fn a_container_emptied_earlier_in_the_same_call_is_removed() {
    let held = upstream(
        vec![project("p1", None), project("p2", None)],
        vec![section("s1", "p1")],
        vec![
            item("t1", Some("s1"), None),
            item("t2", Some("s1"), Some("t1")),
        ],
    );
    let moved_out = with_fields(
        mutation(
            "update",
            2,
            Some("i:t1"),
            Some(task(&"2".repeat(40), "p2/", "t1", false)),
        ),
        &["path"],
    );
    let mutations = vec![
        removal(1, "i:t2"),
        moved_out,
        removal(3, "s:s1"),
        removal(4, "p:p1"),
    ];
    let api = FakeTodoist::new(held);
    let response = push(&api, mutations).unwrap();
    assert!(
        response.results.iter().all(|r| r.ok()),
        "{:?}",
        response.results
    );
    assert_eq!(
        api.recorded.borrow().types(),
        vec![
            "item_delete",
            "item_move",
            "section_delete",
            "project_delete"
        ]
    );
}

#[test]
fn a_container_that_gets_a_task_earlier_in_the_same_call_is_not_removed() {
    let held = upstream(vec![project("p1", None)], vec![], vec![]);
    let created = mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "p1/", "kid", false)),
    );
    let api = FakeTodoist::new(held);
    let response = push(&api, vec![created, removal(2, "p:p1")]).unwrap();
    assert!(response.results[0].ok());
    assert!(
        response.results[1]
            .why()
            .unwrap_or_default()
            .contains("1 task")
    );
    assert_eq!(api.recorded.borrow().types(), vec!["item_add"]);
}
