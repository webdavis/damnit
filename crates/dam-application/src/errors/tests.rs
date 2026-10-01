use super::*;
use dam_domain::{LabelViolation, Oid};

fn oid() -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(1))
}

const HAND_COUNTED_RULES: usize = 19;

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
    }
}

fn one_sample_of_every_refusal() -> Vec<Refusal> {
    vec![
        Refusal::Blocked {
            oid: oid(),
            blockers: vec![],
        },
        Refusal::Cycle {
            oid: oid(),
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
        Refusal::NotATask(oid()),
        Refusal::NotAnEvent(oid()),
        Refusal::NotCompleted(oid()),
        Refusal::NotCommitted(oid()),
        Refusal::DirtyOnPull { oid: oid() },
        Refusal::MoveInsideItself(oid()),
        Refusal::NothingToCommit,
        Refusal::NeedsAnAnswer,
        Refusal::NeedsAnEditor,
        Refusal::UnresolvedConflicts(2),
        Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into(),
        },
        Refusal::StaleRemote {
            remote: "gcal".into(),
            age: None,
            limit: 60,
        },
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
