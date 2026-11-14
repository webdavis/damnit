use super::{oid, remote};
use crate::ports::AnswerRepository;

pub fn answer_repository_contract(store: &dyn AnswerRepository) {
    let here = remote("here");
    let there = remote("there");
    assert_eq!(store.answers(&here, &oid(1)).unwrap(), 0);
    store.count_answer(&here, &oid(1)).unwrap();
    store.count_answer(&here, &oid(1)).unwrap();
    assert_eq!(store.answers(&here, &oid(1)).unwrap(), 2);
    assert_eq!(
        store.answers(&there, &oid(1)).unwrap(),
        0,
        "a count belongs to one remote"
    );
    assert_eq!(
        store.answers(&here, &oid(2)).unwrap(),
        0,
        "a count belongs to one object"
    );
}
