use crate::support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn send(sb: &Sandbox, held: &str, subject: &str, session: &str) -> serde_json::Value {
    let meta = format!("session={session}");
    let line = [
        "new", subject, "--id", "build", "--meta", &meta, "-o", "json",
    ];
    answer(&sb.output_holding(held, &line), "sent.schema.json")
}

#[test]
fn the_same_id_from_another_session_names_another_task() {
    let _guard = support::guard("the_same_id_from_another_session_names_another_task");
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "tool",
        1,
        vec![grant(&["task:read", "task:create", "task:change"])],
    );
    let first = send(&sb, &held, "Fix the build", "a");
    let other = send(&sb, &held, "Fix the build", "b");
    assert_eq!(other["answer"], "new");
    assert_ne!(other["oid"], first["oid"]);
    let again = send(&sb, &held, "Fix the build today", "a");
    assert_eq!(again["answer"], "changed");
    assert_eq!(again["oid"], first["oid"]);
}
