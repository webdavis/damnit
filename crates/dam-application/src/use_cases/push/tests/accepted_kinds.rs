use dam_domain::Kind;

use super::super::*;
use super::{clock, config, launcher};
use crate::ports::Repositories;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, MemoryStore};

#[test]
fn a_push_records_the_kinds_the_remote_accepts() {
    let store = MemoryStore::new();
    let (l, _) = launcher(|_| vec![]);
    push(
        Repositories::of(&store),
        &l,
        &EchoCredentials,
        &clock(),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        store.accepted_kinds(&RemoteName("todoist".into())).unwrap(),
        Some(vec![Kind::Task])
    );
}
