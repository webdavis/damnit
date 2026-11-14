use crate::ports::{HelperError, RemoteHelper};
use crate::remote::RemoteMutation;
use crate::use_cases::push::reconcile::{Settlement, reconcile};
use crate::use_cases::push::removals::Removals;

#[derive(Default)]
pub(super) struct Sent {
    pub(super) settlement: Settlement,
    pub(super) mutations: usize,
    pub(super) held: usize,
    pub(super) stopped_by: Option<HelperError>,
}

pub(super) fn send_in_waves(
    helper: &mut dyn RemoteHelper,
    mut removals: Removals,
    mut pending: Vec<RemoteMutation>,
) -> Result<Sent, HelperError> {
    let mut sent = Sent::default();
    while !pending.is_empty() {
        let wave = removals.wave(pending);
        sent.held += wave.held.len();
        sent.settlement.failed.extend(wave.held);
        pending = wave.waiting;
        if wave.ready.is_empty() {
            continue;
        }
        let outcomes = match helper.push(wave.ready.clone()) {
            Ok(outcomes) => outcomes,
            Err(stop) if sent.mutations == 0 => return Err(stop),
            Err(stop) => {
                let unanswered: Vec<_> = wave.ready.into_iter().chain(pending).collect();
                sent.settlement.absorb(reconcile(&unanswered, vec![]));
                sent.stopped_by = Some(stop);
                break;
            }
        };
        sent.mutations += wave.ready.len();
        let settled = reconcile(&wave.ready, outcomes);
        removals.record(&wave.ready, &settled);
        sent.settlement.absorb(settled);
    }
    Ok(sent)
}
