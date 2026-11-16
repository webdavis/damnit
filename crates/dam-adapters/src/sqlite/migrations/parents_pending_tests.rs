use rusqlite::params;

use super::super::codec::object_to_json;
use super::parents_tests::*;

const STAGE_BEFORE: &str = "SELECT before_json FROM stage WHERE oid = ?1";
const STAGE_AFTER: &str = "SELECT after_json FROM stage WHERE oid = ?1";
const OURS: &str = "SELECT ours_json FROM conflicts WHERE oid = ?1";
const THEIRS: &str = "SELECT theirs_json FROM conflicts WHERE oid = ?1";
const ITEM_AFTER: &str = "SELECT after_json FROM items WHERE target = ?1";

#[test]
fn a_staged_move_gets_the_parent_of_each_side() {
    let conn = version_fifteen_store();
    let places = [task(1, "errands", "Home"), task(2, "chores", "Home")];
    let before = task(3, "milk", "Home/errands");
    let after = task(3, "milk", "Home/chores");
    committed(
        &conn,
        &[places[0].clone(), places[1].clone(), before.clone()],
    );
    working(
        &conn,
        &[places[0].clone(), places[1].clone(), after.clone()],
    );
    conn.execute(
        "INSERT INTO stage (oid, op, before_json, after_json) VALUES (?1, 'update', ?2, ?3)",
        params![
            oid(3).to_string(),
            object_to_json(&before).unwrap(),
            object_to_json(&after).unwrap()
        ],
    )
    .unwrap();

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, STAGE_BEFORE, 3), Some(oid(1)));
    assert_eq!(parent(&conn, STAGE_AFTER, 3), Some(oid(2)));
}

#[test]
fn a_conflict_gets_the_working_parent_for_ours_and_the_remote_parent_for_theirs() {
    let conn = version_fifteen_store();
    let places = [task(1, "errands", "Home"), task(2, "chores", "Home")];
    let ours = task(3, "milk", "Home/errands");
    let theirs = task(3, "milk", "Home/chores");
    everywhere(&conn, &[places[0].clone(), places[1].clone(), ours.clone()]);
    seen(
        &conn,
        "todoist",
        &[places[0].clone(), places[1].clone(), theirs.clone()],
    );
    conn.execute(
        "INSERT INTO conflicts (oid, remote, ours_json, theirs_json) VALUES (?1, 'todoist', ?2, ?3)",
        params![
            oid(3).to_string(),
            object_to_json(&ours).unwrap(),
            object_to_json(&theirs).unwrap()
        ],
    )
    .unwrap();

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, OURS, 3), Some(oid(1)));
    assert_eq!(parent(&conn, THEIRS, 3), Some(oid(2)));
}

#[test]
fn a_token_item_gets_the_parent_its_working_place_names() {
    let conn = version_fifteen_store();
    everywhere(&conn, &[task(1, "errands", "Home")]);
    conn.execute(
        "INSERT INTO items (hash, token, key_id, channel, op, target, before_json, after_json, created, outcome) \
         VALUES ('h', 't', 'k', 'c', 'create', ?1, NULL, ?2, '1970-01-01T00:00:00Z', 'pending')",
        params![
            oid(3).to_string(),
            object_to_json(&task(3, "milk", "Home/errands")).unwrap()
        ],
    )
    .unwrap();

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, ITEM_AFTER, 3), Some(oid(1)));
}

#[test]
fn a_follow_on_gets_the_parent_of_each_copy_it_keeps() {
    let conn = version_fifteen_store();
    let milk = object_to_json(&task(3, "milk", "Home/errands")).unwrap();
    everywhere(&conn, &[task(1, "errands", "Home")]);
    conn.execute(
        "INSERT INTO follow_ons (hash, position, moment, working_json, op, before_json, after_json) \
         VALUES ('h', 0, 'm', ?1, 'update', ?1, ?1)",
        params![milk],
    )
    .unwrap();

    let conn = upgraded(conn);

    for column in ["working_json", "before_json", "after_json"] {
        let query = format!("SELECT {column} FROM follow_ons WHERE hash = 'h' AND ?1 IS NOT NULL");
        assert_eq!(parent(&conn, &query, 3), Some(oid(1)), "{column}");
    }
}
