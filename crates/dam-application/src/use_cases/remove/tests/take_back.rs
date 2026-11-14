use std::collections::BTreeSet;

use dam_domain::{CallerId, ItemHash, Oid, Path, Permission, Priority};

use crate::access::Holder;
use crate::ports::ObjectRepository;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder};
use crate::use_cases::items::land::Landed;
use crate::use_cases::new_object::NewTask;
use crate::use_cases::remove::{ItemAndTask, Removal};

pub(super) fn dentist() -> CallerId {
    CallerId::parse("dentist").unwrap()
}

pub(super) fn dentist_task() -> NewTask {
    NewTask {
        subject: "call the dentist".into(),
        path: Path::parse("inbox").unwrap(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: BTreeSet::new(),
        body: String::new(),
        id: Some(dentist()),
    }
}

pub(super) fn tool(permissions: &[Permission]) -> Holder {
    holder(vec![allowed(permissions, Some(&["inbox"]))])
}

pub(super) fn as_holder<'a>(s: &'a Scenario, holder: &'a Holder) -> Dam<'a> {
    Dam {
        access: holder,
        ..s.dam()
    }
}

pub(super) fn sent(s: &Scenario, holder: &Holder) -> (ItemHash, Oid) {
    match as_holder(s, holder).new_task(dentist_task()).unwrap() {
        Landed::Sent(sent) => (sent.item.hash.clone(), sent.item.target().clone()),
        Landed::Written(object) => panic!("written: {object:?}"),
    }
}

pub(super) fn added(s: &Scenario, holder: &Holder) -> Oid {
    let (hash, oid) = sent(s, holder);
    s.dam().accept(&hash).unwrap();
    oid
}

pub(super) fn waiting_removal(removal: Removal) -> ItemAndTask {
    match removal {
        Removal::Waiting(waiting) => waiting,
        other => panic!("not waiting: {other:?}"),
    }
}

pub(super) fn moved_home(s: &Scenario, oid: &Oid) {
    let mut moved = s.store.get(oid).unwrap().unwrap();
    moved.base_mut().path = Path::parse("home").unwrap();
    s.store.put(&moved).unwrap();
}

mod by_id;
mod by_oid;
