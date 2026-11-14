use dam_protocol::{Mutation, PushResponse};

use super::fixtures::{mutation, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn two_projects_with_a_section_each() -> FakeTodoist {
    FakeTodoist::new(serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [
            {"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false},
            {"id": "p2", "name": "Home", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}
        ],
        "sections": [
            {"id": "s1", "name": "Now", "project_id": "p1", "is_deleted": false},
            {"id": "s2", "name": "Later", "project_id": "p2", "is_deleted": false}
        ],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": "s1", "priority": 1, "checked": false, "is_deleted": false}]
    }))
}

fn move_to(n: u8, remote_id: &str, path: &str, subject: &str) -> Mutation {
    let object = task(&n.to_string().repeat(40), path, subject, false);
    with_fields(
        mutation("update", n, Some(remote_id), Some(object)),
        &["path"],
    )
}

fn create(n: u8, path: &str, subject: &str) -> Mutation {
    let object = task(&n.to_string().repeat(40), path, subject, false);
    mutation("create", n, None, Some(object))
}

fn verdicts(response: &PushResponse) -> Vec<Option<&str>> {
    response.results.iter().map(|r| r.why()).collect()
}

fn parent_of(api: &FakeTodoist, content: &str) -> serde_json::Value {
    let recorded = api.recorded.borrow();
    let added = recorded
        .executed
        .iter()
        .find(|c| c["type"] == "item_add" && c["args"]["content"] == content)
        .unwrap_or_else(|| panic!("no item_add for {content} in {:?}", recorded.types()));
    added["args"]["parent_id"].clone()
}

#[test]
fn a_subtask_filed_this_push_moves_with_its_parent() {
    let api = two_projects_with_a_section_each();
    let mutations = vec![
        create(1, "Work/Now/milk/", "oat"),
        move_to(2, "i:i1", "Home/Later/", "milk"),
        create(3, "Work/Now/", "milk"),
        create(4, "Work/Now/milk/oat/", "Ada"),
        create(5, "Home/Later/milk/oat/", "errands"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(
        verdicts(&response),
        vec![
            None,
            None,
            None,
            Some("no Todoist task named \"oat\" under Work/Now/milk/"),
            None
        ]
    );
    assert_eq!(parent_of(&api, "errands"), "new1");
}

#[test]
fn a_task_filed_and_moved_this_push_is_not_found_at_its_old_place() {
    let api = two_projects_with_a_section_each();
    let mutations = vec![
        create(1, "Work/Now/", "tool"),
        move_to(2, "i:new1", "Home/Later/", "tool"),
        create(3, "Work/Now/tool/", "Ada"),
        create(4, "Home/Later/tool/", "errands"),
    ];
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    assert_eq!(
        verdicts(&response),
        vec![
            None,
            None,
            Some("no Todoist task named \"tool\" under Work/Now/"),
            None
        ]
    );
    assert_eq!(parent_of(&api, "errands"), "new1");
}
