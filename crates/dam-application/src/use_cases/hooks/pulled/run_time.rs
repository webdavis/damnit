use dam_domain::Timestamp;
use jiff::SignedDuration;

use crate::ports::Clock;

const ONE_PULLS_HOOK_RUN: SignedDuration = SignedDuration::from_secs(60);
const SAID_TO_BE_RUNNING_AFTER: SignedDuration = SignedDuration::from_secs(2);

pub(super) struct RunTime<'a> {
    clock: &'a dyn Clock,
    started: Timestamp,
    said_long: bool,
}

impl RunTime<'_> {
    pub(super) fn started_on(clock: &dyn Clock) -> RunTime<'_> {
        RunTime {
            clock,
            started: clock.now(),
            said_long: false,
        }
    }

    pub(super) fn ran_out(&self) -> bool {
        self.spent() >= ONE_PULLS_HOOK_RUN
    }

    pub(super) fn first_time_long(&mut self) -> bool {
        let long = !self.said_long && self.spent() >= SAID_TO_BE_RUNNING_AFTER;
        self.said_long |= long;
        long
    }

    fn spent(&self) -> SignedDuration {
        self.clock.now().duration_since(self.started)
    }
}
