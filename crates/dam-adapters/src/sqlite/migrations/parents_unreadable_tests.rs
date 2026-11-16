use rusqlite::params;

use super::parents_tests::*;
use super::*;

#[test]
fn an_unreadable_object_stops_the_upgrade_naming_its_table_and_id_and_changes_nothing() {
    let mut conn = version_fifteen_store();
    everywhere(&conn, &[task(1, "errands", "Home")]);
    conn.execute(
        "INSERT INTO objects (oid, path, json) VALUES (?1, 'Home', '{\"broken\": true}')",
        params![oid(7).to_string()],
    )
    .unwrap();

    let error = migrate_above_user_version_in_one_transaction(&mut conn)
        .unwrap_err()
        .to_string();

    assert!(error.contains("objects"), "{error}");
    assert!(error.contains(&oid(7).to_string()), "{error}");
    let version: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 15);
}
