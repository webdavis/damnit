use std::collections::BTreeSet;

use dam_domain::Permission::{PendingRead, TaskCreate};
use dam_domain::{ItemHash, Outcome, Path, Priority};

use crate::access::Holder;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder};
use crate::use_cases::items::land::Landed;
use crate::use_cases::items::refusal::ItemRefusal;
use crate::use_cases::new_object::NewTask;

fn dentist_from_tool(s: &Scenario) -> ItemHash {
    let tool = holder(vec![allowed(&[TaskCreate], None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let task = NewTask {
        subject: "call the dentist".into(),
        path: Path::root(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: BTreeSet::new(),
        body: String::new(),
        id: None,
    };
    match dam.new_task(task).unwrap() {
        Landed::Sent(sent) => sent.item.hash,
        Landed::Written(object) => panic!("written: {object:?}"),
    }
}

#[test]
fn the_owner_declines_an_item_and_it_is_never_offered_again() {
    let s = Scenario::default().drawing_from(7);
    let hash = dentist_from_tool(&s);
    let declined = s.dam().dismiss(&hash).unwrap();
    assert!(matches!(declined.outcome, Outcome::Declined(_)));
    assert!(s.dam().waiting_items().unwrap().is_empty());
    assert_eq!(
        s.dam().dismiss(&hash),
        Err(ItemRefusal::NotWaiting {
            hash,
            outcome: "declined"
        }
        .into())
    );
}

#[test]
fn a_token_withdraws_its_own_item_and_not_another_senders() {
    let s = Scenario::default().drawing_from(7);
    let hash = dentist_from_tool(&s);
    let claude = Holder {
        name: "claude".into(),
        ..holder(vec![allowed(&[PendingRead], None)])
    };
    let as_claude = Dam {
        access: &claude,
        ..s.dam()
    };
    assert_eq!(
        as_claude.dismiss(&hash),
        Err(ItemRefusal::NotYours(hash.clone()).into())
    );
    let tool = holder(vec![]);
    let as_tool = Dam {
        access: &tool,
        ..s.dam()
    };
    let withdrawn = as_tool.dismiss(&hash).unwrap();
    assert!(matches!(withdrawn.outcome, Outcome::Withdrawn(_)));
    assert_eq!(
        as_tool.remove(hash.oid()).err(),
        Some(UseCaseError::Refused(Refusal::NotOpenToTokens("rm")))
    );
}
