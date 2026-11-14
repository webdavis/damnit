use dam_protocol::{Mutation, PushResponse};

use super::fixtures::{mutation, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn work_with_a_section_and_two_tasks() -> FakeTodoist {
    FakeTodoist::new(serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [{"id": "s1", "name": "Now", "project_id": "p1", "is_deleted": false}],
        "items": [
            {"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false},
            {"id": "i2", "content": "bread", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}
        ]
    }))
}

fn rename(n: u8, remote_id: &str, path: &str, subject: &str) -> Mutation {
    let object = task(&n.to_string().repeat(40), path, subject, false);
    with_fields(
        mutation("update", n, Some(remote_id), Some(object)),
        &["subject"],
    )
}

fn create(n: u8, path: &str, subject: &str) -> Mutation {
    let object = task(&n.to_string().repeat(40), path, subject, false);
    mutation("create", n, None, Some(object))
}

fn verdicts(response: &PushResponse) -> Vec<Option<&str>> {
    response.results.iter().map(|r| r.why()).collect()
}

fn filed(api: &FakeTodoist, content: &str) -> serde_json::Value {
    let recorded = api.recorded.borrow();
    let added = recorded
        .executed
        .iter()
        .find(|c| c["type"] == "item_add" && c["args"]["content"] == content)
        .unwrap_or_else(|| panic!("no item_add for {content} in {:?}", recorded.types()));
    added["args"].clone()
}

#[test]
fn a_task_created_under_a_renamed_project_is_filed_in_it() {
    let api = work_with_a_section_and_two_tasks();
    let mutations = vec![
        rename(1, "p:p1", "", "Office"),
        create(2, "Office/", "tool"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(verdicts(&response), vec![None, None]);
    assert_eq!(filed(&api, "tool")["project_id"], "p1");
}

#[test]
fn a_task_created_under_a_renamed_section_is_filed_in_it() {
    let api = work_with_a_section_and_two_tasks();
    let mutations = vec![
        rename(1, "s:s1", "Work/", "Later"),
        create(2, "Work/Later/", "tool"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(verdicts(&response), vec![None, None]);
    assert_eq!(filed(&api, "tool")["section_id"], "s1");
}

#[test]
fn a_task_created_or_moved_under_a_renamed_parent_task_is_filed_under_it() {
    let api = work_with_a_section_and_two_tasks();
    let moved = task(&"3".repeat(40), "Work/oat/", "bread", false);
    let mutations = vec![
        rename(1, "i:i1", "Work/", "oat"),
        create(2, "Work/oat/", "tool"),
        with_fields(mutation("update", 3, Some("i:i2"), Some(moved)), &["path"]),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(verdicts(&response), vec![None, None, None]);
    assert_eq!(filed(&api, "tool")["parent_id"], "i1");
    assert_eq!(
        api.recorded.borrow().command("item_move")["args"],
        serde_json::json!({"id": "i2", "parent_id": "i1"})
    );
}

#[test]
fn the_old_name_of_a_renamed_project_no_longer_finds_it() {
    let api = work_with_a_section_and_two_tasks();
    let mutations = vec![rename(1, "p:p1", "", "Office"), create(2, "Work/", "tool")];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(
        verdicts(&response),
        vec![None, Some("no Todoist project named \"Work\"")]
    );
}

#[test]
fn a_task_filed_this_push_is_not_found_under_its_projects_old_name() {
    let api = work_with_a_section_and_two_tasks();
    let mutations = vec![
        create(1, "Work/", "tool"),
        rename(2, "p:p1", "", "Office"),
        create(3, "", "Work"),
        create(4, "Work/tool/", "Ada"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(
        verdicts(&response),
        vec![
            None,
            None,
            None,
            Some("no Todoist task named \"tool\" under Work/")
        ]
    );
}

#[test]
fn a_rename_todoist_refuses_leaves_the_old_name_in_place() {
    let api = work_with_a_section_and_two_tasks().refusing("project_update");
    let mutations = vec![
        rename(1, "p:p1", "", "Office"),
        create(2, "Office/", "tool"),
        create(3, "Work/", "Ada"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert!(!response.results[0].ok());
    assert_eq!(
        response.results[1].why(),
        Some("no Todoist project named \"Office\"")
    );
    assert!(response.results[2].ok(), "{:?}", response.results[2].why());
    assert_eq!(filed(&api, "Ada")["project_id"], "p1");
}
