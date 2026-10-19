use super::*;

fn failure(code: std::ffi::c_int) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(code),
        Some("database is locked".into()),
    )
}

#[test]
fn children_of_the_root_are_found_and_deeper_objects_are_not() {
    let store = SqliteStore::in_memory().unwrap();
    let put = |path: &str, byte: u8| {
        let mut t = dam_domain::Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(byte)), "milk");
        t.base.path = Path::parse(path).unwrap();
        store.put(&Object::Task(t)).unwrap();
    };
    put("", 1);
    put("work", 2);
    put("work/deep", 3);
    let root: Vec<String> = store
        .children_of(&Path::parse("").unwrap())
        .unwrap()
        .iter()
        .map(|o| o.base().path.as_str().to_string())
        .collect();
    assert_eq!(root, vec!["work/".to_string()]);
    let under_work: Vec<String> = store
        .children_of(&Path::parse("work").unwrap())
        .unwrap()
        .iter()
        .map(|o| o.base().path.as_str().to_string())
        .collect();
    assert_eq!(under_work, vec!["work/deep/".to_string()]);
}

#[test]
fn the_upper_bound_sits_above_every_path_under_the_prefix() {
    assert_eq!(upper_bound_of_every_text_starting_with("work/"), "work0");
    assert!("work/a" < upper_bound_of_every_text_starting_with("work/").as_str());
    assert!("work/z/deep/" < upper_bound_of_every_text_starting_with("work/").as_str());
    assert!("work0" >= upper_bound_of_every_text_starting_with("work/").as_str());
}

#[test]
fn a_busy_or_locked_store_is_its_own_outcome() {
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_BUSY)),
        StoreError::Busy(_)
    ));
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_LOCKED)),
        StoreError::Busy(_)
    ));
}

#[test]
fn every_other_store_failure_stays_a_plain_failure() {
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_CORRUPT)),
        StoreError::Failed(_)
    ));
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_FULL)),
        StoreError::Failed(_)
    ));
    assert!(matches!(
        sql(rusqlite::Error::QueryReturnedNoRows),
        StoreError::Failed(_)
    ));
}
