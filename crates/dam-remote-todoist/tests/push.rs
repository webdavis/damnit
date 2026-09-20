mod loopback;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use dam_protocol::{Mutation, WireObject, WireTask};
use dam_remote_todoist::api::TodoistApi;
use dam_remote_todoist::push::push;

fn sync_body() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}]
    })
}

fn sync_body_with_two_projects() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [
            {"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false},
            {"id": "p2", "name": "Home", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}
        ],
        "sections": [],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}]
    })
}

/// A key of the shape dam mints: a UUID whose last character is left free for
/// the helper to number a mutation's commands in.
fn key(n: u8) -> String {
    format!("0102030a-0b0c-4d0e-8f10-1112131415{n:x}0")
}

fn mutation(op: &str, n: u8, remote_id: Option<&str>, object: Option<WireObject>) -> Mutation {
    Mutation {
        op: op.into(),
        oid: format!("{n:x}").repeat(40),
        idempotency_key: key(n),
        remote_id: remote_id.map(str::to_string),
        object,
        fields: vec![],
    }
}

fn with_fields(mut m: Mutation, fields: &[&str]) -> Mutation {
    m.fields = fields.iter().map(|f| f.to_string()).collect();
    m
}

fn task(oid: &str, path: &str, subject: &str, done: bool) -> WireObject {
    WireObject {
        oid: oid.into(),
        remote_id: None,
        kind: "task".into(),
        subject: subject.into(),
        body: "b".into(),
        path: path.into(),
        labels: vec!["errand".into()],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done,
            priority: 1,
            due: Some("2026-09-25".into()),
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

/// An object whose `due` and `recurrence` are both gone, so an update naming
/// only "due" sends the one field that clears the date.
fn task_with_dates_cleared(oid: &str) -> WireObject {
    let mut t = task(oid, "Work/", "unused", false);
    t.task = Some(WireTask {
        done: false,
        priority: 1,
        due: None,
        deadline: None,
        event: None,
    });
    t
}

/// What the fake service did, as opposed to what it was asked to do.
#[derive(Default)]
struct Recorded {
    /// One entry per command actually executed, in order.
    executed: Vec<serde_json::Value>,
    /// Every command uuid ever seen, executed or deduplicated.
    uuids: HashSet<String>,
    issued: u32,
}

impl Recorded {
    fn types(&self) -> Vec<&str> {
        self.executed
            .iter()
            .filter_map(|c| c["type"].as_str())
            .collect()
    }

    fn command(&self, kind: &str) -> serde_json::Value {
        self.executed
            .iter()
            .find(|c| c["type"] == kind)
            .unwrap_or_else(|| panic!("no {kind} in {:?}", self.types()))
            .clone()
    }
}

/// A stand-in for Todoist's sync endpoint that remembers command uuids the way
/// the documented one does: "Todoist will not execute a command that has same
/// UUID as a previously executed command." `refuse` names a command type it
/// answers with an error object instead of executing.
fn todoist(
    tree: serde_json::Value,
    refuse: Option<&'static str>,
) -> (loopback::Loopback, Arc<Mutex<Recorded>>) {
    let state = Arc::new(Mutex::new(Recorded::default()));
    let inner = Arc::clone(&state);
    let server = loopback::serve_with(move |seen| {
        let Some(commands) = seen.body.get("commands").and_then(|c| c.as_array()) else {
            return loopback::Reply::new(200, tree.clone());
        };
        let mut recorded = inner.lock().unwrap();
        let mut status = serde_json::Map::new();
        let mut mapping = serde_json::Map::new();
        for c in commands {
            let uuid = c["uuid"].as_str().unwrap_or_default().to_string();
            let kind = c["type"].as_str().unwrap_or_default();
            if !recorded.uuids.insert(uuid.clone()) {
                status.insert(uuid, "ok".into());
                continue;
            }
            if refuse == Some(kind) {
                status.insert(
                    uuid,
                    serde_json::json!({"error_code": 15, "error": "refused by the test"}),
                );
                continue;
            }
            recorded.executed.push(c.clone());
            status.insert(uuid, "ok".into());
            if let Some(temp) = c["temp_id"].as_str() {
                recorded.issued += 1;
                mapping.insert(temp.to_string(), format!("new{}", recorded.issued).into());
            }
        }
        loopback::Reply::new(
            200,
            serde_json::json!({"sync_status": status, "temp_id_mapping": mapping}),
        )
    });
    (server, state)
}

#[test]
fn creates_updates_and_deletes_become_the_right_commands() {
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![
        mutation(
            "create",
            1,
            None,
            Some(task(&"1".repeat(40), "Work/", "eggs", false)),
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "milk", true)),
            ),
            &["done", "subject"],
        ),
        mutation("delete", 3, Some("p:p1"), None),
        mutation(
            "create",
            4,
            None,
            Some(task(&"4".repeat(40), "Nope/", "lost", false)),
        ),
        with_fields(
            mutation(
                "update",
                5,
                Some("i:i3"),
                Some(task_with_dates_cleared(&"5".repeat(40))),
            ),
            &["due"],
        ),
        with_fields(
            mutation(
                "update",
                6,
                Some("i:i4"),
                Some(task_with_dates_cleared(&"6".repeat(40))),
            ),
            &["deadline"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert_eq!(response.results.len(), 6);
    assert!(response.results[0].ok);
    assert_eq!(response.results[0].remote_id.as_deref(), Some("i:new1"));
    assert!(response.results[1].ok, "{:?}", response.results[1].why);
    assert!(response.results[2].ok);
    assert!(!response.results[3].ok);
    assert!(response.results[3].why.as_deref().unwrap().contains("Nope"));
    assert!(response.results[4].ok);
    assert!(response.results[5].ok);

    let recorded = recorded.lock().unwrap();
    let add = recorded.command("item_add");
    assert_eq!(add["args"]["content"], "eggs");
    assert_eq!(add["args"]["project_id"], "p1");
    assert_eq!(add["args"]["priority"], 4, "dam p1 is API 4");
    assert_eq!(
        add["args"]["due"],
        serde_json::json!({"date": "2026-09-25"})
    );
    assert_eq!(add["args"]["labels"], serde_json::json!(["errand"]));
    assert!(
        add["args"].get("deadline").is_none(),
        "a create with no deadline omits the field rather than clearing one"
    );
    let update = recorded.command("item_update");
    assert_eq!(update["args"]["id"], "i1");
    assert_eq!(update["args"]["content"], "milk");
    assert!(
        update["args"].get("priority").is_none(),
        "only changed fields are sent"
    );
    assert!(recorded.types().contains(&"item_close"));
    assert!(recorded.types().contains(&"project_delete"));
    let cleared: Vec<&serde_json::Value> = recorded
        .executed
        .iter()
        .filter(|c| c["type"] == "item_update")
        .collect();
    assert_eq!(
        cleared[1]["args"],
        serde_json::json!({"id": "i3", "due": null})
    );
    assert_eq!(
        cleared[2]["args"],
        serde_json::json!({"id": "i4", "deadline": null})
    );
}

/// Finding 1.1: the request left, the answer never arrived, dam resent. The
/// resent mutation carries the key it carried the first time, so the service
/// recognises it and creates nothing twice.
#[test]
fn a_resent_mutation_repeats_its_uuids_and_creates_nothing_twice() {
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let first = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let again = first.clone();

    let one = push(&api, first).unwrap();
    assert!(one.results[0].ok);
    let two = push(&api, again).unwrap();

    let recorded = recorded.lock().unwrap();
    assert_eq!(
        recorded
            .types()
            .iter()
            .filter(|t| **t == "item_add")
            .count(),
        1,
        "the resend was deduplicated: {:?}",
        recorded.types()
    );
    assert_eq!(
        recorded.uuids.len(),
        1,
        "the resend repeated the uuid rather than minting one"
    );
    assert!(
        !two.results[0].ok,
        "a resend dam cannot learn an id from is not reported as pushed"
    );
}

/// Every command of one mutation gets its own uuid, or the second is dropped
/// as a duplicate of the first.
#[test]
fn the_commands_of_one_mutation_have_distinct_uuids() {
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", true)),
    )];
    assert!(push(&api, mutations).unwrap().results[0].ok);
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["item_add", "item_close"]);
    assert_eq!(recorded.uuids.len(), 2);
}

/// One command's error is that mutation's failure and nobody else's.
#[test]
fn a_refused_command_fails_only_its_own_mutation() {
    let (server, recorded) = todoist(sync_body(), Some("item_update"));
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![
        mutation(
            "create",
            1,
            None,
            Some(task(&"1".repeat(40), "Work/", "eggs", false)),
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
        mutation("delete", 3, Some("p:p1"), None),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok);
    assert!(!response.results[1].ok);
    assert_eq!(
        response.results[1].why.as_deref(),
        Some("refused by the test")
    );
    assert!(response.results[2].ok);
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["item_add", "project_delete"]);
}

/// A failure reading the tree is not a per-mutation result: no mutation was
/// attempted, so there is nothing to report against one.
#[test]
fn a_failure_reading_the_tree_stops_the_push_before_any_mutation() {
    let mut routes = HashMap::new();
    routes.insert("POST /sync", (500, serde_json::json!({"error": "boom"})));
    let server = loopback::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let why = push(&api, mutations).unwrap_err();
    assert!(why.contains("500"), "{why}");
    assert_eq!(
        server.seen.lock().unwrap().len(),
        1,
        "nothing was attempted after the read failed"
    );
}

#[test]
fn a_path_change_moves_the_task_through_the_move_command() {
    let (server, recorded) = todoist(sync_body_with_two_projects(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![with_fields(
        mutation(
            "update",
            9,
            Some("i:i1"),
            Some(task(&"9".repeat(40), "Home/", "milk", false)),
        ),
        &["path"],
    )];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok, "{:?}", response.results[0].why);
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["item_move"]);
    assert_eq!(
        recorded.command("item_move")["args"],
        serde_json::json!({"id": "i1", "project_id": "p2"})
    );
}

#[test]
fn a_path_and_subject_change_moves_first_then_updates_the_content() {
    let (server, recorded) = todoist(sync_body_with_two_projects(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![with_fields(
        mutation(
            "update",
            8,
            Some("i:i1"),
            Some(task(&"8".repeat(40), "Home/", "oat", false)),
        ),
        &["path", "subject"],
    )];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok, "{:?}", response.results[0].why);
    let recorded = recorded.lock().unwrap();
    assert_eq!(
        recorded.types(),
        vec!["item_move", "item_update"],
        "move takes exactly one container, so it goes on its own"
    );
    assert_eq!(
        recorded.command("item_move")["args"],
        serde_json::json!({"id": "i1", "project_id": "p2"})
    );
    assert_eq!(recorded.command("item_update")["args"]["content"], "oat");
}

/// The stored `remote_id` is Todoist's own string on the way back out. One
/// carrying path syntax must never reach a command.
#[test]
fn a_hostile_remote_id_is_refused_and_never_reaches_a_command() {
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![
        mutation("delete", 1, Some("i:../../projects/p1"), None),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1?force=true"),
                Some(task(&"2".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    for result in &response.results {
        assert!(!result.ok, "{result:?}");
        assert!(
            result
                .why
                .as_deref()
                .unwrap()
                .contains("unusable Todoist id"),
            "{result:?}"
        );
    }
    assert!(recorded.lock().unwrap().executed.is_empty());
}

/// A rate limit is not one mutation's problem. Reporting it per mutation would
/// leave dam marking the rest of the batch pushed while Todoist took none of
/// it, so the push stops and says how long to wait.
#[test]
fn a_rate_limit_stops_the_push_and_names_the_retry_time() {
    let server = loopback::serve_with(|seen| {
        if seen.body.get("commands").is_none() {
            return loopback::Reply::new(200, sync_body());
        }
        loopback::Reply::new(429, serde_json::json!({"error": "Rate limit exceeded"}))
            .with_header("Retry-After", "42")
    });
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let why = push(&api, mutations).unwrap_err();
    assert!(why.contains("rate limiting"), "{why}");
    assert!(why.contains("42s"), "{why}");
}

/// The other half of the same decision. Once mutations are being applied, a
/// failure belongs to the mutation it happened on: the ones Todoist already
/// took are reported as taken, and dam marks exactly those pushed. That is
/// only safe because a resend repeats its key, so anything dam sends again is
/// deduplicated rather than duplicated.
#[test]
fn a_failure_part_way_through_belongs_to_its_own_mutation() {
    let writes = Arc::new(Mutex::new(0u32));
    let counted = Arc::clone(&writes);
    let server = loopback::serve_with(move |seen| {
        if seen.body.get("commands").is_none() {
            return loopback::Reply::new(200, sync_body());
        }
        let mut n = counted.lock().unwrap();
        *n += 1;
        if *n == 2 {
            return loopback::Reply::new(500, serde_json::json!({"error": "upstream is down"}));
        }
        let uuids: Vec<String> = seen.body["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["uuid"].as_str().unwrap_or_default().to_string())
            .collect();
        let status: serde_json::Map<String, serde_json::Value> = uuids
            .into_iter()
            .map(|u| (u, serde_json::Value::from("ok")))
            .collect();
        loopback::Reply::new(
            200,
            serde_json::json!({"sync_status": status, "temp_id_mapping": {}}),
        )
    });
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![
        with_fields(
            mutation(
                "update",
                1,
                Some("i:i1"),
                Some(task(&"1".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "oat", false)),
            ),
            &["subject"],
        ),
        with_fields(
            mutation(
                "update",
                3,
                Some("i:i1"),
                Some(task(&"3".repeat(40), "Work/", "soy", false)),
            ),
            &["subject"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok);
    assert!(!response.results[1].ok);
    assert!(response.results[1].why.as_deref().unwrap().contains("500"));
    assert!(
        response.results[2].ok,
        "the batch goes on after one mutation's failure"
    );
}
