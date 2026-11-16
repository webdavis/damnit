use super::oid;
use crate::config::HookEvent::{TaskChanged, TaskNew};
use crate::ports::HookMarkRepository;

pub fn hook_mark_repository_contract(store: &dyn HookMarkRepository) {
    assert_eq!(store.hooked(&oid(1)).unwrap(), None);
    store.mark_hooked(&oid(1), TaskNew).unwrap();
    store.mark_hooked(&oid(2), TaskNew).unwrap();
    assert_eq!(store.hooked(&oid(1)).unwrap(), Some(TaskNew));
    store.mark_hooked(&oid(1), TaskChanged).unwrap();
    assert_eq!(
        store.hooked(&oid(1)).unwrap(),
        Some(TaskChanged),
        "the latest hook replaces the earlier one"
    );
    store.forget_hooked(&oid(1)).unwrap();
    store.forget_hooked(&oid(3)).unwrap();
    assert_eq!(store.hooked(&oid(1)).unwrap(), None);
    assert_eq!(
        store.hooked(&oid(2)).unwrap(),
        Some(TaskNew),
        "a mark belongs to one object"
    );
}
