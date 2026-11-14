use dam_domain::Path;

use super::Removals;
use crate::remote::{MutationOp, RemoteMutation};
use crate::testing::oid;

fn removal(of: u8) -> RemoteMutation {
    RemoteMutation {
        oid: oid(of),
        op: MutationOp::Delete,
        idempotency_key: String::new(),
        remote_id: None,
        object: None,
        fields: vec![],
    }
}

#[test]
fn a_wave_with_nothing_to_send_holds_the_removals_that_wait_on_each_other() {
    let shared = Path::parse("Home/P").unwrap();
    let mut removals = Removals::default();
    for each in [1, 2] {
        removals.folders.insert(oid(each), shared.clone());
        removals.upstream.insert(oid(each), shared.clone());
    }
    let wave = removals.wave(vec![removal(1), removal(2)]);
    assert!(wave.ready.is_empty());
    assert!(wave.waiting.is_empty());
    let held: Vec<_> = wave.held.iter().map(|(oid, _)| oid.clone()).collect();
    assert_eq!(held, vec![oid(1), oid(2)]);
    assert!(wave.held.iter().all(|(_, why)| !why.is_empty()));
}
