use dam_protocol::{Mutation, MutationResult, Verdict};

use super::fixtures::{mutation, task};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn upstream() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [{"id": "p1", "name": "Home", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [
            {"id": "s1", "name": "errands", "project_id": "p1", "is_deleted": false},
            {"id": "s2", "name": "errands", "project_id": "p1", "is_deleted": false},
            {"id": "s3", "name": "jobs", "project_id": "p1", "is_deleted": false}
        ],
        "items": [
            {"id": "r1", "content": "P", "project_id": "p1", "section_id": "s3", "parent_id": null, "priority": 1, "checked": false, "is_deleted": false},
            {"id": "r2", "content": "P", "project_id": "p1", "section_id": "s3", "parent_id": null, "priority": 1, "checked": false, "is_deleted": false}
        ]
    })
}

fn create(n: u8, path: &str, subject: &str) -> Mutation {
    mutation(
        "create",
        n,
        None,
        Some(task(&format!("{n:x}").repeat(40), path, subject, false)),
    )
}

fn into_remote(mut m: Mutation, container: &str) -> Mutation {
    m.container_remote_id = Some(container.into());
    m
}

fn into_created(mut m: Mutation, n: u8) -> Mutation {
    m.container_oid = Some(format!("{n:x}").repeat(40));
    m
}

fn pushed(mutations: Vec<Mutation>) -> (Vec<MutationResult>, Vec<serde_json::Value>) {
    let api = FakeTodoist::new(upstream());
    let response = push(&api, mutations, &mut |_| {}).unwrap();
    let executed = api.recorded.borrow().executed.clone();
    (response.results, executed)
}

fn added(executed: &[serde_json::Value], name: &str) -> serde_json::Value {
    executed
        .iter()
        .find(|c| c["args"]["content"] == name || c["args"]["name"] == name)
        .unwrap_or_else(|| panic!("nothing added named {name} in {executed:?}"))["args"]
        .clone()
}

fn refused(result: &MutationResult) -> &str {
    match &result.verdict {
        Verdict::Refused { why, .. } => why,
        other => panic!("not refused: {other:?}"),
    }
}

#[test]
fn a_task_created_in_each_of_two_same_named_sections_lands_in_the_one_it_names() {
    let (results, executed) = pushed(vec![
        into_remote(create(1, "Home/errands/", "stamps"), "s:s1"),
        into_remote(create(2, "Home/errands/", "parcel"), "s:s2"),
    ]);
    assert!(results.iter().all(|r| r.ok()), "{results:?}");
    assert_eq!(added(&executed, "stamps")["section_id"], "s1");
    assert_eq!(added(&executed, "parcel")["section_id"], "s2");
}

#[test]
fn a_task_created_under_a_named_task_lands_under_that_one_of_its_namesakes() {
    let (results, executed) = pushed(vec![into_remote(create(1, "Home/jobs/P/", "kid"), "i:r2")]);
    assert!(results.iter().all(|r| r.ok()), "{results:?}");
    let kid = added(&executed, "kid");
    assert_eq!(
        (kid["section_id"].as_str(), kid["parent_id"].as_str()),
        (Some("s3"), Some("r2"))
    );
}

#[test]
fn a_section_created_in_a_named_project_is_added_to_that_project() {
    let (results, executed) = pushed(vec![
        into_remote(create(1, "Home/", "chores"), "p:p1"),
        into_created(create(2, "Home/chores/", "kid"), 1),
    ]);
    assert!(results.iter().all(|r| r.ok()), "{results:?}");
    assert_eq!(executed[0]["type"], "section_add");
    assert_eq!(added(&executed, "chores")["project_id"], "p1");
}

#[test]
fn a_child_lands_in_the_section_this_push_created_beside_a_same_named_one() {
    let (results, executed) = pushed(vec![
        into_remote(create(1, "Home/", "errands"), "p:p1"),
        into_created(create(2, "Home/errands/", "kid"), 1),
    ]);
    assert!(results.iter().all(|r| r.ok()), "{results:?}");
    assert_eq!(added(&executed, "kid")["section_id"], "new1");
}

#[test]
fn a_child_lands_under_the_task_this_push_created_beside_a_same_named_one() {
    let (results, executed) = pushed(vec![
        into_remote(create(1, "Home/jobs/", "P"), "s:s3"),
        into_created(create(2, "Home/jobs/P/", "kid"), 1),
    ]);
    assert!(results.iter().all(|r| r.ok()), "{results:?}");
    assert_eq!(added(&executed, "kid")["parent_id"], "new1");
}

#[test]
fn a_child_of_a_parent_this_push_did_not_create_is_refused_and_nothing_is_sent() {
    let (results, executed) = pushed(vec![into_created(create(2, "Home/errands/", "kid"), 1)]);
    assert_eq!(
        refused(&results[0]),
        "its parent 1111111 is not in Todoist yet; run: dam push"
    );
    assert!(executed.is_empty(), "{executed:?}");
}

#[test]
fn a_named_container_todoist_no_longer_has_fails_and_nothing_is_sent() {
    let (results, executed) = pushed(vec![into_remote(
        create(1, "Home/errands/", "stamps"),
        "s:gone",
    )]);
    assert_eq!(
        refused(&results[0]),
        "Todoist no longer has s:gone; run: dam pull, then dam push"
    );
    assert!(executed.is_empty(), "{executed:?}");
}

#[test]
fn a_name_two_sections_share_is_refused_when_none_is_named() {
    let (results, executed) = pushed(vec![create(3, "Home/errands/", "stamps")]);
    assert_eq!(
        refused(&results[0]),
        "2 Todoist sections are named \"errands\" under Home/ and dam named neither; \
         run: dam pull, then dam mv 3333333 --parent OID"
    );
    assert!(executed.is_empty(), "{executed:?}");
}

#[test]
fn a_name_two_tasks_share_is_refused_when_none_is_named() {
    let (results, executed) = pushed(vec![create(3, "Home/jobs/P/", "kid")]);
    assert_eq!(
        refused(&results[0]),
        "2 Todoist tasks are named \"P\" under Home/jobs/ and dam named neither; \
         run: dam pull, then dam mv 3333333 --parent OID"
    );
    assert!(executed.is_empty(), "{executed:?}");
}

#[test]
fn a_delete_removes_the_id_it_names_and_never_a_namesake() {
    let (results, executed) = pushed(vec![mutation("delete", 1, Some("i:r2"), None)]);
    assert!(results.iter().all(|r| r.ok()), "{results:?}");
    assert_eq!(executed.len(), 1, "{executed:?}");
    assert_eq!(
        (
            executed[0]["type"].as_str(),
            executed[0]["args"]["id"].as_str()
        ),
        (Some("item_delete"), Some("r2"))
    );
}
