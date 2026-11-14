use crate::errors::UseCaseError;
use crate::ports::{HelperError, RemoteHelper};
use crate::remote::{MutationVerdict, PushOutcome, RemoteMutation};
use crate::use_cases::push::reconcile::{Settlement, reconcile};
use crate::use_cases::push::removals::Removals;

#[derive(Default)]
pub(super) struct Sent {
    pub(super) settlement: Settlement,
    pub(super) mutations: usize,
    pub(super) answered: usize,
    pub(super) held: usize,
    pub(super) skipped: usize,
    pub(super) stopped_by: Option<HelperError>,
    pub(super) stopped_before_any_answer: bool,
}

impl Sent {
    pub(super) fn absorb(&mut self, later: Sent) {
        self.settlement.absorb(later.settlement);
        self.mutations += later.mutations;
        self.answered += later.answered;
        self.held += later.held;
        self.skipped += later.skipped;
        self.stopped_by = later.stopped_by;
    }
}

pub(super) fn send_in_waves(
    helper: &mut dyn RemoteHelper,
    mut removals: Removals,
    mut pending: Vec<RemoteMutation>,
    record_before_the_call: &dyn Fn(&[RemoteMutation]) -> Result<(), UseCaseError>,
) -> Result<Sent, UseCaseError> {
    let mut sent = Sent::default();
    while !pending.is_empty() {
        let wave = removals.wave(pending);
        sent.held += wave.held.len();
        sent.settlement.failed.extend(wave.held);
        pending = wave.waiting;
        if wave.ready.is_empty() {
            continue;
        }
        record_before_the_call(&wave.ready)?;
        let answer = helper
            .push(wave.ready.clone())
            .unwrap_or_else(|stop| PushOutcome {
                outcomes: vec![],
                stopped_by: Some(stop),
            });
        sent.stopped_before_any_answer =
            answer.stopped_by.is_some() && sent.mutations == 0 && !answered_anything(&answer);
        sent.mutations += wave.ready.len();
        let settled = reconcile(&wave.ready, answer.outcomes);
        sent.answered += wave.ready.len().saturating_sub(settled.unanswered.len());
        removals.record(&wave.ready, &settled);
        sent.settlement.absorb(settled);
        if let Some(stop) = answer.stopped_by {
            sent.settlement.absorb(reconcile(&pending, vec![]));
            sent.stopped_by = Some(stop);
            break;
        }
    }
    Ok(sent)
}

fn answered_anything(answer: &PushOutcome) -> bool {
    answer
        .outcomes
        .iter()
        .any(|o| !matches!(o.verdict, MutationVerdict::Unanswered { .. }))
}
