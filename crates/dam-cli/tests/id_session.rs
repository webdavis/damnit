mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn send(sb: &Sandbox, held: &str, subject: &str, session: &str) -> serde_json::Value {
    let meta = format!("session={session}");
    let line = ["new", subject, "--id", "build", "--meta", &meta, "--json"];
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

#[test]
fn the_owners_ids_are_kept_apart_by_session_and_show_names_each() {
    let _guard = support::guard("the_owners_ids_are_kept_apart_by_session_and_show_names_each");
    let sb = Sandbox::new();
    let mut oids = Vec::new();
    for session in ["a", "b"] {
        let meta = format!("session={session}");
        let (ok, out, err) = sb.dam(&[
            "new",
            "Fix the build",
            "--id",
            "build",
            "--meta",
            &meta,
            "--json",
        ]);
        assert!(ok, "{err}");
        let made: serde_json::Value = serde_json::from_str(&out).unwrap();
        oids.push(made["oid"].as_str().unwrap().to_string());
    }
    assert_ne!(oids[0], oids[1]);
    for oid in &oids {
        let (ok, out, err) = sb.dam(&["show", oid, "--json", "--no-pull"]);
        assert!(ok, "{err}");
        let shown: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(shown["id"], "build");
    }
}
