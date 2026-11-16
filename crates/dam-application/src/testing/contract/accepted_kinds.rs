use dam_domain::{Field, Kind};

use super::remote;
use crate::ports::AcceptedKindsRepository;

pub fn accepted_kinds_repository_contract(store: &dyn AcceptedKindsRepository) {
    let todoist = remote("todoist");
    let gcal = remote("gcal");
    assert_eq!(store.accepted_kinds(&todoist).unwrap(), None);
    store.set_accepted_kinds(&todoist, &[Kind::Task]).unwrap();
    store.set_accepted_kinds(&gcal, &[]).unwrap();
    assert_eq!(
        store.accepted_kinds(&todoist).unwrap(),
        Some(vec![Kind::Task])
    );
    assert_eq!(store.accepted_kinds(&gcal).unwrap(), Some(vec![]));
    store
        .set_accepted_kinds(&todoist, &[Kind::Task, Kind::Event])
        .unwrap();
    assert_eq!(
        store.accepted_kinds(&todoist).unwrap(),
        Some(vec![Kind::Task, Kind::Event])
    );
    assert_eq!(store.accepted_fields(&gcal).unwrap(), None);
    store
        .set_accepted_fields(&todoist, &[Field::Subject, Field::Due])
        .unwrap();
    assert_eq!(
        store.accepted_fields(&todoist).unwrap(),
        Some(vec![Field::Subject, Field::Due])
    );
    assert_eq!(
        store.accepted_kinds(&todoist).unwrap(),
        Some(vec![Kind::Task, Kind::Event])
    );
    store.set_accepted_fields(&gcal, &[]).unwrap();
    assert_eq!(store.accepted_fields(&gcal).unwrap(), Some(vec![]));
}
