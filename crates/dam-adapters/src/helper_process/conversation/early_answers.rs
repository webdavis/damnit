use std::time::{Duration, Instant};

use dam_application::{HelperError, PushOutcome};
use dam_protocol::{EarlyAnswer, MutationResult, PushResponse};
use serde_json::Value;

use super::ProcessHelper;
use super::requests::reply_from;
use crate::wire::push_from_wire;

const LONGEST_READ_AFTER_AN_INTERRUPT: Duration = Duration::from_millis(200);

enum Line {
    Early(MutationResult),
    Final(PushResponse),
}

fn read(line: Value) -> Result<Line, HelperError> {
    if line.get("early_answer").is_none() {
        return reply_from(line).map(Line::Final);
    }
    serde_json::from_value::<EarlyAnswer>(line)
        .map(|early| Line::Early(early.early_answer))
        .map_err(|e| HelperError::Protocol(e.to_string()))
}

fn stopped(response: PushResponse, stop: HelperError) -> PushOutcome {
    PushOutcome {
        stopped_by: Some(stop),
        ..push_from_wire(response)
    }
}

fn early_only(results: Vec<MutationResult>) -> PushResponse {
    PushResponse {
        results,
        stopped: None,
    }
}

impl ProcessHelper {
    pub(super) fn read_answers_as_they_come(&mut self) -> PushOutcome {
        let mut early = Vec::new();
        let stop = loop {
            match self.receive_within_the_deadline().and_then(read) {
                Ok(Line::Early(result)) => early.push(result),
                Ok(Line::Final(response)) => return push_from_wire(response),
                Err(HelperError::Cancelled) => {
                    return self.keep_what_came_before_the_interrupt(early);
                }
                Err(stop) => break stop,
            }
        };
        if !self.killed {
            self.kill_and_reap();
        }
        stopped(early_only(early), stop)
    }

    fn keep_what_came_before_the_interrupt(
        &mut self,
        mut early: Vec<MutationResult>,
    ) -> PushOutcome {
        let budget_ends = Instant::now() + LONGEST_READ_AFTER_AN_INTERRUPT;
        while let Some(line) = self.next_line_before(budget_ends) {
            match read(line) {
                Ok(Line::Early(result)) => early.push(result),
                Ok(Line::Final(response)) => return stopped(response, HelperError::Cancelled),
                Err(_) => break,
            }
        }
        stopped(early_only(early), HelperError::Cancelled)
    }

    fn next_line_before(&self, budget_ends: Instant) -> Option<Value> {
        let left = budget_ends.checked_duration_since(Instant::now())?;
        self.answers.recv_timeout(left).ok()?.ok()?
    }
}
