use dam_domain::Timestamp;
use jiff::SignedDuration;

use crate::ports::Clock;

const ONE_PULLS_HOOK_RUN: SignedDuration = SignedDuration::from_secs(60);

pub(super) struct RunTime<'a> {
    clock: &'a dyn Clock,
    started: Timestamp,
}

impl RunTime<'_> {
    pub(super) fn started_on(clock: &dyn Clock) -> RunTime<'_> {
        RunTime {
            clock,
            started: clock.now(),
        }
    }

    pub(super) fn ran_out(&self) -> bool {
        self.spent() >= ONE_PULLS_HOOK_RUN
    }

    fn spent(&self) -> SignedDuration {
        self.clock.now().duration_since(self.started)
    }
}
