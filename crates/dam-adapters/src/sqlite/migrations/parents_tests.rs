use dam_domain::{Object, Oid, Path, Task};
use rusqlite::params;

use super::super::codec::{object_from_json, object_to_json};
use super::*;

pub(super) fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

pub(super) fn task(b: u8, subject: &str, path: &str) -> Object {
    let mut t = Task::new(oid(b), subject);
    t.base.path = Path::parse(path).unwrap();
    Object::Task(t)
}

fn json(o: &Object) -> String {
    object_to_json(o).unwrap()
}

pub(super) fn version_fifteen_store() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    for (_, batch) in BATCHES.into_iter().filter(|(version, _)| *version <= 15) {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", 15u32).unwrap();
    conn
}

pub(super) fn working(conn: &Connection, objects: &[Object]) {
    for o in objects {
        conn.execute(
            "INSERT INTO objects (oid, path, json) VALUES (?1, ?2, ?3)",
            params![o.oid().to_string(), o.base().path.to_string(), json(o)],
        )
        .unwrap();
    }
}

pub(super) fn committed(conn: &Connection, objects: &[Object]) {
    for o in objects {
        conn.execute(
            "INSERT INTO committed (oid, json) VALUES (?1, ?2)",
            params![o.oid().to_string(), json(o)],
        )
        .unwrap();
    }
}

pub(super) fn seen(conn: &Connection, remote: &str, objects: &[Object]) {
    for o in objects {
        conn.execute(
            "INSERT INTO snapshots (remote, oid, json) VALUES (?1, ?2, ?3)",
            params![remote, o.oid().to_string(), json(o)],
        )
        .unwrap();
    }
}

pub(super) fn everywhere(conn: &Connection, objects: &[Object]) {
    working(conn, objects);
    committed(conn, objects);
}

pub(super) fn upgraded(mut conn: Connection) -> Connection {
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    conn
}

pub(super) fn parent(conn: &Connection, query: &str, b: u8) -> Option<Oid> {
    let stored: String = conn
        .query_row(query, [oid(b).to_string()], |r| r.get(0))
        .unwrap_or_else(|e| panic!("{query}: {e}"));
    object_from_json(&stored).unwrap().base().parent.clone()
}

pub(super) const WORKING: &str = "SELECT json FROM objects WHERE oid = ?1";
pub(super) const COMMITTED: &str = "SELECT json FROM committed WHERE oid = ?1";
pub(super) const SEEN: &str = "SELECT json FROM snapshots WHERE remote = 'todoist' AND oid = ?1";

#[test]
fn a_task_inside_one_object_gets_that_object_as_its_parent() {
    let conn = version_fifteen_store();
    everywhere(
        &conn,
        &[task(1, "errands", "Home"), task(2, "milk", "Home/errands")],
    );

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, WORKING, 2), Some(oid(1)));
    assert_eq!(parent(&conn, COMMITTED, 2), Some(oid(1)));
    assert_eq!(parent(&conn, WORKING, 1), None);
}

#[test]
fn a_task_inside_two_same_named_objects_keeps_no_parent() {
    let conn = version_fifteen_store();
    everywhere(
        &conn,
        &[
            task(1, "errands", "Home"),
            task(2, "errands", "Home"),
            task(3, "milk", "Home/errands"),
        ],
    );

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, WORKING, 3), None);
    assert_eq!(parent(&conn, COMMITTED, 3), None);
}

#[test]
fn a_task_in_a_plain_place_below_an_object_gets_that_object() {
    let conn = version_fifteen_store();
    everywhere(
        &conn,
        &[
            task(1, "errands", "Home"),
            task(2, "milk", "Home/errands/later"),
        ],
    );

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, WORKING, 2), Some(oid(1)));
}

#[test]
fn a_remote_copy_elsewhere_gets_its_parent_from_the_remote_copies() {
    let conn = version_fifteen_store();
    let errands = task(1, "errands", "Home");
    let chores = task(2, "chores", "Home");
    everywhere(
        &conn,
        &[
            errands.clone(),
            chores.clone(),
            task(3, "milk", "Home/errands"),
        ],
    );
    seen(
        &conn,
        "todoist",
        &[errands, chores, task(3, "milk", "Home/chores")],
    );

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, SEEN, 3), Some(oid(2)));
    assert_eq!(parent(&conn, COMMITTED, 3), Some(oid(1)));
}

#[test]
fn a_half_committed_removal_leaves_the_task_without_a_parent_where_namesakes_were_seen() {
    let conn = version_fifteen_store();
    let first = task(1, "errands", "Home");
    seen(
        &conn,
        "todoist",
        &[
            first.clone(),
            task(2, "errands", "Home"),
            task(3, "milk", "Home/errands"),
        ],
    );
    committed(&conn, &[first.clone(), task(3, "milk", "Home/errands")]);
    working(&conn, &[first, task(3, "milk", "Home")]);

    let conn = upgraded(conn);

    assert_eq!(parent(&conn, SEEN, 3), None);
    assert_eq!(parent(&conn, COMMITTED, 3), None);
    assert_eq!(parent(&conn, WORKING, 3), None);
}

#[test]
fn a_second_upgrade_leaves_every_parent_as_the_first_set_it() {
    let conn = version_fifteen_store();
    everywhere(
        &conn,
        &[task(1, "errands", "Home"), task(2, "milk", "Home/errands")],
    );
    let mut conn = upgraded(conn);
    let first: String = conn
        .query_row(WORKING, [oid(2).to_string()], |r| r.get(0))
        .unwrap();

    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();

    let second: String = conn
        .query_row(WORKING, [oid(2).to_string()], |r| r.get(0))
        .unwrap();
    assert_eq!(second, first);
}

pub(super) fn a_stored_task() -> String {
    json(&task(9, "milk", "work"))
}
