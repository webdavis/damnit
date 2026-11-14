use dam_domain::Permission::TaskCreate;
use dam_domain::{ItemHash, Path, Priority};

use super::Reopened;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder};
use crate::use_cases::items::land::Landed;
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::new_object::NewTask;
use crate::use_cases::prefix::Named;

fn added_dentist(s: &Scenario) -> ItemHash {
    let tool = holder(vec![allowed(&[TaskCreate], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let landed = dam
        .new_task(NewTask {
            subject: "call the dentist".into(),
            path: Path::parse("inbox").unwrap(),
            priority: Priority::default(),
            due: None,
            deadline: None,
            labels: Default::default(),
            body: String::new(),
            id: None,
        })
        .unwrap();
    let Landed::Sent(sent) = landed else {
        panic!("written: {landed:?}");
    };
    s.dam().add(&[Named::Item(sent.item.hash.clone())]).unwrap();
    sent.item.hash
}

#[test]
fn reopening_an_added_item_whose_change_was_committed_is_refused() {
    let s = Scenario::default().drawing_from(7);
    let hash = added_dentist(&s);
    s.dam().commit("dentist").unwrap();

    let refused = s.dam().reopen(&hash).unwrap_err();

    assert!(
        matches!(
            &refused,
            UseCaseError::Refused(Refusal::Item(ItemRefusal::AlreadyCommitted(h))) if *h == hash
        ),
        "{refused:?}"
    );
}

#[test]
fn reopening_an_added_item_still_staged_unstages_its_object() {
    let s = Scenario::default().drawing_from(7);
    let hash = added_dentist(&s);

    let reopened = s.dam().reopen(&hash).unwrap();

    assert_eq!(reopened, Reopened::Unstaged(hash.oid().clone()));
    assert!(s.store.staged().unwrap().is_empty());
}
