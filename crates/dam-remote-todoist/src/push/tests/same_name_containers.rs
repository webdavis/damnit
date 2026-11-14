use dam_protocol::Mutation;

use super::fixtures::{mutation, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

const HASH_ORDERS_TRIED: usize = 32;

fn project(id: &str, name: &str) -> serde_json::Value {
    serde_json::json!({"id": id, "name": name, "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false})
}

fn section(id: &str, name: &str, project_id: &str) -> serde_json::Value {
    serde_json::json!({"id": id, "name": name, "project_id": project_id, "is_deleted": false})
}

fn item(id: &str, content: &str, parent_id: Option<&str>) -> serde_json::Value {
    serde_json::json!({"id": id, "content": content, "project_id": "p1", "section_id": "s1", "parent_id": parent_id, "priority": 1, "checked": false, "is_deleted": false})
}

fn held(
    projects: Vec<serde_json::Value>,
    sections: Vec<serde_json::Value>,
    items: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({"sync_token": "t", "full_sync": true, "projects": projects, "sections": sections, "items": items})
}

fn create(n: u8, path: &str, subject: &str) -> Mutation {
    mutation(
        "create",
        n,
        None,
        Some(task(&n.to_string().repeat(40), path, subject, false)),
    )
}

fn sent_whichever_order_the_tree_holds_them(
    upstream: &serde_json::Value,
    mutations: &[Mutation],
) -> Vec<Vec<serde_json::Value>> {
    (0..HASH_ORDERS_TRIED)
        .map(|_| {
            let api = FakeTodoist::new(upstream.clone());
            let response = push(&api, mutations.to_vec()).unwrap();
            assert!(
                response.results.iter().all(|r| r.ok()),
                "{:?}",
                response.results
            );
            api.recorded.borrow().executed.clone()
        })
        .collect()
}

fn args_of(executed: &[serde_json::Value], kind: &str, id_or_content: &str) -> serde_json::Value {
    executed
        .iter()
        .find(|c| {
            c["type"] == kind
                && (c["args"]["content"] == id_or_content || c["args"]["id"] == id_or_content)
        })
        .unwrap_or_else(|| panic!("no {kind} for {id_or_content} in {executed:?}"))["args"]
        .clone()
}

#[test]
fn a_task_created_under_a_new_namesake_of_a_held_task_names_the_new_ones_id() {
    let upstream = held(
        vec![project("p1", "Work")],
        vec![section("s1", "Now", "p1")],
        vec![
            item("r1", "P", None),
            item("r2", "Q", Some("r1")),
            item("r3", "moved", None),
        ],
    );
    let mutations = [
        create(1, "Work/Now/", "P"),
        create(2, "Work/Now/P/", "Q"),
        create(3, "Work/Now/P/Q/", "kid"),
        with_fields(
            mutation(
                "update",
                4,
                Some("i:r3"),
                Some(task(&"4".repeat(40), "Work/Now/P/", "moved", false)),
            ),
            &["path"],
        ),
    ];
    for executed in sent_whichever_order_the_tree_holds_them(&upstream, &mutations) {
        assert_eq!(args_of(&executed, "item_add", "Q")["parent_id"], "new1");
        assert_eq!(args_of(&executed, "item_add", "kid")["parent_id"], "new2");
        assert_eq!(args_of(&executed, "item_move", "r3")["parent_id"], "new1");
    }
}

#[test]
fn a_task_created_under_a_new_namesake_of_a_held_section_names_the_new_ones_id() {
    let upstream = held(
        vec![project("p1", "Work")],
        vec![section("r1", "P", "p1")],
        vec![],
    );
    let mutations = [create(1, "Work/", "P"), create(2, "Work/P/", "kid")];
    for executed in sent_whichever_order_the_tree_holds_them(&upstream, &mutations) {
        assert_eq!(executed[0]["type"], "section_add");
        assert_eq!(args_of(&executed, "item_add", "kid")["section_id"], "new1");
    }
}

#[test]
fn a_task_created_under_a_new_namesake_of_a_held_project_names_the_new_ones_id() {
    let upstream = held(vec![project("r1", "P")], vec![], vec![]);
    let mutations = [create(1, "", "P"), create(2, "P/", "kid")];
    for executed in sent_whichever_order_the_tree_holds_them(&upstream, &mutations) {
        assert_eq!(executed[0]["type"], "project_add");
        assert_eq!(args_of(&executed, "item_add", "kid")["project_id"], "new1");
    }
}

#[test]
fn a_task_under_a_container_this_push_did_not_create_still_finds_it_by_name() {
    let upstream = held(
        vec![project("p1", "Work")],
        vec![section("s1", "Now", "p1")],
        vec![item("r1", "P", None)],
    );
    let mutations = [create(1, "Work/Now/P/", "kid")];
    for executed in sent_whichever_order_the_tree_holds_them(&upstream, &mutations) {
        let kid = args_of(&executed, "item_add", "kid");
        assert_eq!(
            (kid["section_id"].as_str(), kid["parent_id"].as_str()),
            (Some("s1"), Some("r1"))
        );
    }
}

fn refiled_naming(container: &str, path: &str) -> Mutation {
    let mut m = with_fields(
        mutation(
            "update",
            9,
            Some("i:c1"),
            Some(task(&"9".repeat(40), path, "c", false)),
        ),
        &["path"],
    );
    m.container_remote_id = Some(container.into());
    m
}

#[test]
fn a_refiled_task_lands_under_the_namesake_task_the_mutation_names() {
    let upstream = held(
        vec![project("p1", "Work")],
        vec![section("s1", "Now", "p1")],
        vec![
            item("r1", "P", None),
            item("r2", "P", None),
            item("c1", "c", Some("r1")),
        ],
    );
    let mutations = [refiled_naming("i:r2", "Work/Now/P/")];
    for executed in sent_whichever_order_the_tree_holds_them(&upstream, &mutations) {
        assert_eq!(args_of(&executed, "item_move", "c1")["parent_id"], "r2");
    }
}

#[test]
fn a_refiled_task_lands_in_the_namesake_section_the_mutation_names() {
    let upstream = held(
        vec![project("p1", "Work")],
        vec![section("s1", "P", "p1"), section("s2", "P", "p1")],
        vec![item("c1", "c", None)],
    );
    let mutations = [refiled_naming("s:s2", "Work/P/")];
    for executed in sent_whichever_order_the_tree_holds_them(&upstream, &mutations) {
        assert_eq!(args_of(&executed, "item_move", "c1")["section_id"], "s2");
    }
}
