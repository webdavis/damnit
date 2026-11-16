use dam_domain::Object;

use super::Removals;
use crate::remote::{MutationOp, RemoteMutation};
use crate::testing::{oid, task_in};

fn removal(of: u8) -> RemoteMutation {
    RemoteMutation {
        oid: oid(of),
        op: MutationOp::Delete,
        idempotency_key: String::new(),
        remote_id: None,
        container_remote_id: None,
        container_oid: None,
        object: None,
        fields: vec![],
    }
}

#[test]
fn a_wave_with_nothing_to_send_holds_the_removals_that_wait_on_each_other() {
    let mut removals = Removals::default();
    for (each, other) in [(1, 2), (2, 1)] {
        removals.removing.insert(oid(each));
        let mut held = task_in(each, "x", "Home/");
        held.base.parent = Some(oid(other));
        removals.upstream.insert(oid(each), Object::Task(held));
    }
    let wave = removals.wave(vec![removal(1), removal(2)]);
    assert!(wave.ready.is_empty());
    assert!(wave.waiting.is_empty());
    let held: Vec<_> = wave.held.iter().map(|hold| hold.oid.clone()).collect();
    assert_eq!(held, vec![oid(1), oid(2)]);
    assert!(wave.held.iter().all(|hold| !hold.why.is_empty()));
}

fn removing_1_with(held: dam_domain::Task) -> Removals {
    let mut removals = Removals::default();
    removals.removing.insert(oid(1));
    removals
        .upstream
        .insert(oid(1), Object::Task(task_in(1, "P", "Home/")));
    removals
        .upstream
        .insert(held.base.oid.clone(), Object::Task(held));
    removals
}

fn under(byte: u8, path: &str, parent: u8) -> dam_domain::Task {
    let mut t = task_in(byte, "c", path);
    t.base.parent = Some(oid(parent));
    t
}

fn held_why(removals: &mut Removals) -> Vec<(dam_domain::Oid, String)> {
    let wave = removals.wave(vec![removal(1)]);
    assert!(wave.ready.is_empty());
    wave.held
        .into_iter()
        .map(|hold| (hold.oid, hold.why))
        .collect()
}

const PLACE_NOT_KNOWN: &str = "is not known upstream; run: dam pull, then dam push";

#[test]
fn a_removal_waits_on_a_task_inside_its_folder_whose_last_seen_parent_is_gone() {
    let mut removals = removing_1_with(under(2, "Home/P/", 9));
    let held = held_why(&mut removals);
    let c = oid(2);
    assert_eq!(
        held,
        vec![(
            oid(1),
            format!("where {} sits {PLACE_NOT_KNOWN}", c.short())
        )]
    );
}

#[test]
fn a_removal_goes_ahead_when_a_task_whose_last_seen_parent_is_gone_sits_elsewhere() {
    let mut removals = removing_1_with(under(2, "Work/", 9));
    let wave = removals.wave(vec![removal(1)]);
    assert_eq!(wave.ready.len(), 1);
}

#[test]
fn a_removal_waits_when_the_last_seen_parents_form_a_cycle() {
    let mut removals = removing_1_with(under(2, "Work/", 3));
    removals
        .upstream
        .insert(oid(3), Object::Task(under(3, "Work/", 2)));
    let held = held_why(&mut removals);
    assert_eq!(held.len(), 1);
    assert!(held[0].1.ends_with(PLACE_NOT_KNOWN), "{held:?}");
}

#[test]
fn a_removal_with_no_last_seen_copy_is_held_until_a_pull() {
    let mut removals = Removals::default();
    removals.removing.insert(oid(1));
    let wave = removals.wave(vec![removal(1)]);
    assert!(wave.ready.is_empty());
    let hold = &wave.held[0];
    assert_eq!(hold.oid, oid(1));
    let why = &hold.why;
    assert!(why.ends_with("run: dam pull, then dam push"), "{why}");
}
