use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Field, Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::{Attempt, Attempts, Declared, HelperError, RemoteHelper, RemoteName, Store};
use crate::remote::{RemoteCapabilities, RemoteMutation};
use crate::use_cases::as_it_stands::as_it_stands;
use crate::use_cases::push::idempotency::key;

pub(super) struct Sending {
    pub(super) attempt: Attempt,
    pub(super) repeats_the_last: bool,
}

pub(super) type Attempting = BTreeMap<Oid, Sending>;

pub(super) fn attempting(
    store: &dyn Store,
    helper: &dyn RemoteHelper,
    remote: &RemoteName,
    record: &Attempts,
    mutations: &mut [RemoteMutation],
) -> Result<Attempting, UseCaseError> {
    let mut attempting = Attempting::new();
    for mutation in mutations {
        let earlier = record.get(&mutation.oid).map_or(&[][..], Vec::as_slice);
        let declared = helper.declared_on_the_wire(mutation)?;
        let answers = store.answers(remote, &mutation.oid)?;
        let sending = next(answers, earlier, mutation, declared);
        mutation.idempotency_key = sending.attempt.key.clone();
        attempting.insert(mutation.oid.clone(), sending);
    }
    Ok(attempting)
}

fn next(answers: u64, earlier: &[Attempt], write: &RemoteMutation, declared: Declared) -> Sending {
    let sends_the_same = |last: &&Attempt| {
        last.op == write.op
            && last.container_remote_id == write.container_remote_id
            && last.declared == declared
    };
    if let Some(last) = earlier.last().filter(sends_the_same) {
        return Sending {
            attempt: last.clone(),
            repeats_the_last: true,
        };
    }
    Sending {
        attempt: Attempt {
            key: key(answers, earlier.len(), write, &declared),
            op: write.op,
            container_remote_id: write.container_remote_id.clone(),
            declared,
        },
        repeats_the_last: false,
    }
}

pub(super) fn differing(
    helper: &dyn RemoteHelper,
    caps: &RemoteCapabilities,
    now: Option<&Object>,
    earlier: &[Attempt],
) -> Result<BTreeSet<Field>, HelperError> {
    let Some(now) = now.filter(|_| !earlier.is_empty()) else {
        return Ok(BTreeSet::new());
    };
    let as_it_stands = as_it_stands(helper, caps, now)?;
    Ok(earlier
        .iter()
        .flat_map(|attempt| &attempt.declared)
        .filter(|(field, sent)| as_it_stands.get(field).is_some_and(|now| now != *sent))
        .map(|(field, _)| *field)
        .collect())
}
