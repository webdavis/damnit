use super::*;
use dam_domain::{Blockers, LabelViolation, Tracked};

use crate::testing::oid;

const HAND_COUNTED_RULES: usize = 43;

fn hand_numbered_slot(refusal: &Refusal) -> usize {
    match refusal {
        Refusal::Blocked { .. } => 0,
        Refusal::Cycle { .. } => 1,
        Refusal::Labels(_) => 2,
        Refusal::UnknownCategory(_) => 3,
        Refusal::NoSuchObject(_) => 4,
        Refusal::NoWorkingObject(_) => 5,
        Refusal::NoSuchRemote(_) => 6,
        Refusal::NotATask(_) => 7,
        Refusal::NotAnEvent(_) => 8,
        Refusal::NotCompleted(_) => 9,
        Refusal::NotCommitted(_) => 10,
        Refusal::DirtyOnPull { .. } => 11,
        Refusal::MoveInsideItself(_) => 12,
        Refusal::NothingToCommit => 13,
        Refusal::NeedsAnAnswer => 14,
        Refusal::NeedsAnEditor => 15,
        Refusal::UnresolvedConflicts(_) => 16,
        Refusal::MissingCredential { .. } => 17,
        Refusal::StaleRemote { .. } => 18,
        Refusal::RemoteExists(_) => 19,
        Refusal::UnreadableRecurrence { .. } => 20,
        Refusal::NotACategoryValue { .. } => 21,
        Refusal::NoConflict(_) => 22,
        Refusal::AmbiguousPrefix { .. } => 23,
        Refusal::Token(_) => 24,
        Refusal::NotPermitted(_) => 25,
        Refusal::NotOpenToTokens(_) => 26,
        Refusal::SubjectNamesNoFolder(_) => 27,
        Refusal::FocusFull { .. } => 28,
        Refusal::TokenExists(_) => 29,
        Refusal::NotGrantable(_) => 30,
        Refusal::NoSuchToken(_) => 31,
        Refusal::Item(ItemRefusal::NothingChanged) => 32,
        Refusal::Item(ItemRefusal::Declined(_)) => 33,
        Refusal::Item(ItemRefusal::NotAnObject(_)) => 34,
        Refusal::Item(ItemRefusal::NotWaiting { .. }) => 35,
        Refusal::Item(ItemRefusal::TargetGone { .. }) => 36,
        Refusal::Item(ItemRefusal::TargetNotCommitted { .. }) => 37,
        Refusal::Item(ItemRefusal::TargetStaged { .. }) => 38,
        Refusal::Item(ItemRefusal::Clash { .. }) => 39,
        Refusal::Item(ItemRefusal::NotYours(_)) => 40,
        Refusal::Item(ItemRefusal::EditedSinceAdded { .. }) => 41,
        Refusal::Item(ItemRefusal::SubjectNamesNoFolder { .. }) => 42,
    }
}

fn hash() -> dam_domain::ItemHash {
    dam_domain::ItemHash::new(oid(1))
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
        Refusal::NotATask(oid(1)),
        Refusal::NotAnEvent(oid(1)),
        Refusal::NotCompleted(oid(1)),
        Refusal::NotCommitted(oid(1)),
        Refusal::DirtyOnPull { oid: oid(1) },
        Refusal::MoveInsideItself(oid(1)),
        Refusal::SubjectNamesNoFolder(oid(1)),
        Refusal::NothingToCommit,
        Refusal::NeedsAnAnswer,
        Refusal::NeedsAnEditor,
        Refusal::UnresolvedConflicts(Some(2)),
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
        Refusal::TokenExists("vpt".into()),
        Refusal::NotGrantable("config:change".into()),
        Refusal::NoSuchToken("vpt".into()),
        Refusal::Item(ItemRefusal::NothingChanged),
        Refusal::Item(ItemRefusal::Declined(hash())),
        Refusal::Item(ItemRefusal::NotAnObject(hash())),
        Refusal::Item(ItemRefusal::NotYours(hash())),
        Refusal::Item(ItemRefusal::NotWaiting {
            hash: hash(),
            outcome: "added",
        }),
        Refusal::Item(ItemRefusal::TargetGone {
            hash: hash(),
            target: oid(2),
        }),
        Refusal::Item(ItemRefusal::TargetNotCommitted {
            hash: hash(),
            target: oid(2),
        }),
        Refusal::Item(ItemRefusal::TargetStaged {
            hash: hash(),
            target: oid(2),
        }),
        Refusal::Item(ItemRefusal::Clash {
            hash: hash(),
            target: oid(2),
            fields: vec![dam_domain::Field::Subject],
        }),
        Refusal::Item(ItemRefusal::EditedSinceAdded {
            hash: hash(),
            target: oid(2),
        }),
        Refusal::Item(ItemRefusal::SubjectNamesNoFolder {
            hash: hash(),
            target: oid(2),
        }),
    ]
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

#[test]
fn every_rule_has_its_own_name() {
    let refusals = one_sample_of_every_refusal();
    let mut names: Vec<&str> = refusals.iter().map(Refusal::name).collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "two rules share a name: {names:?}");
}

#[test]
fn every_name_is_lower_snake_case_and_says_something() {
    for refusal in one_sample_of_every_refusal() {
        let name = refusal.name();
        assert!(!name.is_empty(), "{refusal:?}");
        assert!(
            name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "{name} is not snake_case"
        );
    }
}
