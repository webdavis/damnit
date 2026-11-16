use dam_adapters::SqliteStore;
use dam_application::testing::oid;
use dam_application::{ObjectRepository, RemoteName, RemoteTrackingRepository};
use dam_domain::{Object, Path, Task};

use super::{FAKE_ONLY, FAKE_TOKEN, Files, Helpers, rows_of};
use crate::args::CheckName;

fn task(byte: u8, subject: &str, path: &str, parent: Option<u8>) -> Object {
    let mut task = Task::new(oid(byte), subject);
    task.base.path = Path::parse(path).unwrap();
    task.base.parent = parent.map(oid);
    Object::Task(task)
}

fn holding(files: &Files, objects: &[Object]) {
    let store = SqliteStore::open(&files.store_path()).unwrap();
    for object in objects {
        store.put(object).unwrap();
    }
}

fn parents(files: &Files) -> Vec<String> {
    let helpers = Helpers::default();
    let rows = rows_of(&files.report(&helpers, &[CheckName::Parents]));
    assert!(helpers.credentials_given.borrow().is_empty());
    rows
}

fn two_errands_and(more: impl IntoIterator<Item = Object>) -> Vec<Object> {
    let errands = [
        task(1, "errands", "Home", None),
        task(2, "errands", "Home", None),
    ];
    errands.into_iter().chain(more).collect()
}

#[test]
fn a_store_whose_parents_are_all_known_is_well() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    holding(
        &files,
        &[
            task(1, "errands", "Home", None),
            task(3, "milk", "Home/errands/later", Some(1)),
        ],
    );
    assert_eq!(
        parents(&files),
        ["  ✓ parents: dam knows every object's parent"]
    );
}

#[test]
fn an_object_under_two_same_named_objects_is_listed_with_both() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    holding(
        &files,
        &two_errands_and([task(3, "milk", "Home/errands", None)]),
    );
    assert_eq!(
        parents(&files),
        [
            "  ⚠ parents: 1 object whose parent dam does not know",
            "    ⚠ 0303030: milk in Home/errands/, its parent is one of 0101010, 0202020",
            "      → run: dam mv 0303030 --parent <oid>",
        ]
    );
}

#[test]
fn an_object_a_remote_holds_is_settled_by_a_pull() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    holding(
        &files,
        &two_errands_and([task(3, "milk", "Home/errands", None)]),
    );
    let store = SqliteStore::open(&files.store_path()).unwrap();
    let fake = RemoteName("fake".into());
    store.map_remote_id(&fake, &oid(3), "i:3").unwrap();
    drop(store);
    assert_eq!(
        parents(&files)[2],
        "      → run: dam pull, or dam mv 0303030 --parent <oid>"
    );
}

#[test]
fn many_unsettled_objects_show_the_count_and_the_first_few() {
    let files = Files::new(FAKE_ONLY, FAKE_TOKEN);
    let milk = (3..10).map(|n| task(n, "milk", "Home/errands", None));
    holding(&files, &two_errands_and(milk));
    let rows = parents(&files);
    assert_eq!(
        rows[0],
        "  ⚠ parents: 7 objects whose parent dam does not know"
    );
    assert_eq!(
        rows.last().unwrap(),
        "    ⚠ 2 more: shown once these are settled"
    );
}
