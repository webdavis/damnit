use crate::ports::Store;
use dam_domain::{Author, Change};

use super::history::record;
use super::{oid, task, token_author};

pub fn origin_repository_contract(store: &dyn Store) {
    let mut made = record(1, "made", 1);
    made.changes[0].author = token_author();
    let mut pulled_back = record(2, "pulled back", 1);
    pulled_back.pulled = true;
    pulled_back.changes[0].change = Change::Update {
        before: task(1, "made", ""),
        after: task(1, "pulled back", ""),
    };
    let mut pulled_in = record(3, "pulled in", 2);
    pulled_in.pulled = true;
    store.commit(&made).unwrap();
    store.commit(&pulled_back).unwrap();
    store.commit(&pulled_in).unwrap();

    let origins = store.origins().unwrap();
    assert_eq!(
        origins[&oid(1)].author,
        token_author(),
        "an object's origin is its first create, whatever changed it after"
    );
    assert_eq!(
        origins[&oid(2)].author,
        Author::Owner,
        "an object a pull created is the owner's"
    );
}
