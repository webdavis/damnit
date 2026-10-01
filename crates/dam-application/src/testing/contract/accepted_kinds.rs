use dam_domain::Kind;

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
}
