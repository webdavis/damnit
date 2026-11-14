use dam_domain::Permission::{PendingRead, TaskCreate};
use dam_domain::{Change, Channel, Item, Object, Outcome, Timestamp, Token, TokenAuthor};

use crate::access::Holder;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, task_in};

fn sent_by(name: &str, byte: u8, path: &str) -> Item {
    Item::propose(
        TokenAuthor {
            name: name.into(),
            key_id: Token::generate(&mut |b: &mut [u8]| b.fill(byte))
                .key_id()
                .clone(),
            channel: Channel::Local,
        },
        Change::Create {
            after: Object::Task(task_in(byte, "call the dentist", path)),
        },
        Timestamp::UNIX_EPOCH,
        &mut |b| b.fill(byte),
    )
}

fn hashes(dam: &Dam<'_>) -> Vec<String> {
    dam.waiting_items()
        .unwrap()
        .iter()
        .map(|i| i.author.name.clone())
        .collect()
}

#[test]
fn a_token_sees_its_own_waiting_items_and_others_only_with_pending_read() {
    let s = Scenario::default();
    s.store.put_item(&sent_by("tool", 1, "inbox")).unwrap();
    s.store.put_item(&sent_by("codex", 2, "work")).unwrap();
    let settled = sent_by("tool", 3, "inbox");
    s.store.put_item(&settled).unwrap();
    s.store
        .settle(&settled.hash, Outcome::Withdrawn(Timestamp::UNIX_EPOCH))
        .unwrap();
    let as_token = |token: &Holder| {
        hashes(&Dam {
            access: token,
            ..s.dam()
        })
    };
    assert_eq!(
        as_token(&holder(vec![allowed(&[TaskCreate], None)])),
        ["tool"]
    );
    let reviewer = Holder {
        name: "claude".into(),
        ..holder(vec![allowed(&[PendingRead], Some(&["work"]))])
    };
    assert_eq!(as_token(&reviewer), ["codex"]);
    assert_eq!(hashes(&s.dam()), ["tool", "codex"]);
}
