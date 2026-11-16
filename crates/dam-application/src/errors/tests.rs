use super::*;
use dam_domain::{Blockers, LabelViolation, Tracked};

use crate::testing::oid;

mod commit_samples;
mod hook_samples;
mod item_samples;
mod names;
mod parent_samples;
use commit_samples::commit_samples;
use hook_samples::hook_samples;
use item_samples::item_samples;
use parent_samples::parent_samples;

const HAND_COUNTED_RULES: usize = 57;

fn hand_numbered_slot(refusal: &Refusal) -> usize {
    match refusal {
        Refusal::Blocked { .. } => 0,
        Refusal::Cycle { .. } => 1,
        Refusal::Labels(_) | Refusal::ExclusiveAfterMigrate { .. } => 2,
        Refusal::UnknownCategory(_) => 3,
        Refusal::NoSuchObject(_) => 4,
        Refusal::NoWorkingObject(_) => 5,
        Refusal::NoSuchRemote(_) => 6,
        Refusal::NotATask(_) => 7,
        Refusal::NotAnEvent(_) => 8,
        Refusal::NotCompleted(_) => 9,
        Refusal::NotCommitted(_) => 10,
        Refusal::MoveInsideItself(_) => 11,
        Refusal::NothingToCommit => 12,
        Refusal::NeedsAnAnswer => 13,
        Refusal::NeedsAnEditor => 14,
        Refusal::MissingCredential { .. } => 15,
        Refusal::StaleRemote { .. } => 16,
        Refusal::RemoteExists(_) => 17,
        Refusal::UnreadableRecurrence { .. } => 18,
        Refusal::NotACategoryValue { .. } => 19,
        Refusal::NoConflict(_) => 20,
        Refusal::AmbiguousPrefix { .. } => 21,
        Refusal::Token(_) => 22,
        Refusal::NotPermitted(_) => 23,
        Refusal::NotOpenToTokens(_) => 24,
        Refusal::SubjectNamesNoFolder(_) => 25,
        Refusal::FocusFull { .. } => 26,
        Refusal::TokenExists(_) => 27,
        Refusal::NotGrantable(_) => 28,
        Refusal::NoSuchToken(_) => 29,
        Refusal::NoSuchSetting(_) => 30,
        Refusal::Item(ItemRefusal::NothingChanged) => 31,
        Refusal::Item(ItemRefusal::Declined(_)) => 32,
        Refusal::Item(ItemRefusal::NotAnObject(_)) => 33,
        Refusal::Item(ItemRefusal::NotWaiting { .. }) => 34,
        Refusal::Item(ItemRefusal::TargetGone { .. }) => 35,
        Refusal::Item(ItemRefusal::TargetNotCommitted { .. }) => 36,
        Refusal::Item(ItemRefusal::TargetStaged { .. }) => 37,
        Refusal::Item(ItemRefusal::Clash { .. }) => 38,
        Refusal::Item(ItemRefusal::NotYours(_)) => 39,
        Refusal::Item(ItemRefusal::Cycle { .. }) => 40,
        Refusal::Item(ItemRefusal::Blocked { .. }) => 41,
        Refusal::Item(ItemRefusal::EditedSinceAdded { .. }) => 42,
        Refusal::Item(ItemRefusal::SubjectNamesNoFolder { .. }) => 43,
        Refusal::Item(ItemRefusal::RemovesAChangedTask { .. }) => 44,
        Refusal::Item(ItemRefusal::AlreadyCommitted(_)) => 45,
        Refusal::PulledCommit(_) => 46,
        Refusal::NotInCommit { .. } => 47,
        Refusal::HookRefused(_) => 48,
        Refusal::StartedByHook(_) => 49,
        Refusal::ParentNotKnown { .. } => 50,
        Refusal::PathOutsideParent { .. } => 51,
        Refusal::SeveralHoldThePath { .. } => 52,
        Refusal::SyncNotAllowed(_) => 53,
        Refusal::HookNotDeclared(_) => 54,
        Refusal::NoSuchFilter(_) => 55,
        Refusal::CountMismatch { .. } => 56,
    }
}

fn one_sample_of_every_refusal() -> Vec<Refusal> {
    vec![
        Refusal::Blocked {
            oid: oid(1),
            blockers: Blockers {
                children: vec![oid(2)],
                ..Blockers::default()
            },
        },
        Refusal::Cycle {
            oid: oid(1),
            path: vec![],
        },
        Refusal::Labels(LabelViolation::Exclusive {
            category: "effort".into(),
            held: vec![],
        }),
        Refusal::UnknownCategory("effort".into()),
        Refusal::NoSuchObject("abab".into()),
        Refusal::NoWorkingObject("abab".into()),
        Refusal::NoSuchRemote("todoist".into()),
        Refusal::RemoteExists("todoist".into()),
        Refusal::SyncNotAllowed("todoist".into()),
        Refusal::NotATask(oid(1)),
        Refusal::NotAnEvent(oid(1)),
        Refusal::NotCompleted(oid(1)),
        Refusal::NotCommitted(oid(1)),
        Refusal::MoveInsideItself(oid(1)),
        Refusal::SubjectNamesNoFolder(oid(1)),
        Refusal::NothingToCommit,
        Refusal::NeedsAnAnswer,
        Refusal::NeedsAnEditor,
        Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into(),
        },
        Refusal::StaleRemote {
            remote: "gcal".into(),
            age: None,
            limit: 60,
        },
        Refusal::UnreadableRecurrence {
            rule: "every other week".into(),
            tracked: Tracked::No,
        },
        Refusal::NotACategoryValue {
            category: "effort".into(),
            value: "home".into(),
        },
        Refusal::NoConflict(oid(1)),
        Refusal::AmbiguousPrefix {
            text: "abab".into(),
            matches: vec![oid(1).to_string(), oid(2).to_string()],
        },
        Refusal::Token(crate::TokenRefusal::Unknown),
        Refusal::NotPermitted(dam_domain::Permission::TaskRead),
        Refusal::NotOpenToTokens("new"),
        Refusal::FocusFull {
            day: jiff::civil::date(2026, 11, 12),
            focused: vec![oid(1), oid(2), oid(3)],
        },
        Refusal::TokenExists("tool".into()),
        Refusal::NotGrantable("config:change".into()),
        Refusal::NoSuchToken("tool".into()),
        Refusal::NoSuchSetting("nope".into()),
    ]
    .into_iter()
    .chain(commit_samples())
    .chain(hook_samples())
    .chain(parent_samples())
    .chain(item_samples().into_iter().map(Refusal::Item))
    .collect()
}

#[test]
fn every_variant_has_a_sample() {
    let mut slots: Vec<usize> = one_sample_of_every_refusal()
        .iter()
        .map(hand_numbered_slot)
        .collect();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(
        slots,
        (0..HAND_COUNTED_RULES).collect::<Vec<usize>>(),
        "a rule has no sample in one_sample_of_every_refusal, so the checks below never see it"
    );
}

#[test]
fn no_two_samples_share_a_hand_numbered_slot() {
    let samples = one_sample_of_every_refusal();
    let mut slots: Vec<usize> = samples.iter().map(hand_numbered_slot).collect();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(
        slots.len(),
        samples.len(),
        "a new rule reused an existing slot instead of taking a fresh one"
    );
}
