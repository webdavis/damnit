use dam_application::testing::{FakeProgress, Said};

use super::*;

fn a_store_at(version: u32) -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    for batch in [V1, V2, V3, V4, V5, V6, V7, V8, V9, V10] {
        conn.execute_batch(batch).unwrap();
    }
    conn.pragma_update(None, "user_version", version).unwrap();
    conn
}

fn said_while_opening(mut conn: Connection) -> Vec<Said> {
    let progress = FakeProgress::default();
    upgrade(&mut conn, &progress).unwrap();
    progress.said.take()
}

#[test]
fn an_older_store_is_upgraded_as_a_step() {
    let said = said_while_opening(a_store_at(10));
    let upgrading = Said::Started("store_upgrade: upgrading the store ".into(), None);
    assert_eq!(said, [upgrading, Said::Finished]);
}

#[test]
fn a_new_store_and_a_current_store_say_nothing() {
    assert_eq!(
        said_while_opening(Connection::open_in_memory().unwrap()),
        []
    );
    assert_eq!(said_while_opening(upgraded(a_store_at(10))), []);
}

fn upgraded(mut conn: Connection) -> Connection {
    migrate_above_user_version_in_one_transaction(&mut conn).unwrap();
    conn
}
