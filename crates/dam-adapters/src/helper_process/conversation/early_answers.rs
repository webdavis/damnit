use dam_application::{HelperError, PushOutcome};
use dam_protocol::{EarlyAnswer, MutationResult, PushResponse};
use serde_json::Value;

use super::ProcessHelper;
use super::requests::reply_from;
use crate::wire::push_from_wire;

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

impl ProcessHelper {
    pub(super) fn read_answers_as_they_come(&mut self) -> PushOutcome {
        let mut early = Vec::new();
        let stop = loop {
            match self.receive_within_the_deadline().and_then(read) {
                Ok(Line::Early(result)) => early.push(result),
                Ok(Line::Final(response)) => return push_from_wire(response),
                Err(stop) => break stop,
            }
        };
        if !self.killed {
            self.kill_and_reap();
        }
        PushOutcome {
            stopped_by: Some(stop),
            ..push_from_wire(PushResponse {
                results: early,
                stopped: None,
            })
        }
    }
}
