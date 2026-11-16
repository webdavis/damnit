use std::cell::Cell;

use dam_domain::{Date, Timestamp};
use jiff::SignedDuration;

use crate::config::HookEvent;
use crate::ports::{Clock, EditRequest, FunctionFailure, Handed, Histories, Hooks, Pull};
use crate::testing::FakeHooks;

pub struct SlowHooks<'a> {
    pub hooks: &'a FakeHooks,
    pub now: Cell<Timestamp>,
    pub each_call: SignedDuration,
}

impl<'a> SlowHooks<'a> {
    pub fn taking(hooks: &'a FakeHooks, seconds: i64) -> SlowHooks<'a> {
        SlowHooks {
            hooks,
            now: Cell::new(Timestamp::UNIX_EPOCH),
            each_call: SignedDuration::from_secs(seconds),
        }
    }
}

impl Hooks for SlowHooks<'_> {
    fn run(
        &self,
        event: HookEvent,
        task: &Handed,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.later();
        self.hooks.run(event, task, histories)
    }

    fn pulled(
        &self,
        task: &Handed,
        before: Option<&Handed>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.later();
        self.hooks.pulled(task, before, pull)
    }
}

impl SlowHooks<'_> {
    fn later(&self) {
        let later = self.now.get().checked_add(self.each_call);
        self.now.set(later.unwrap_or(Timestamp::MAX));
    }
}

impl Clock for SlowHooks<'_> {
    fn today(&self) -> Date {
        Date::constant(2026, 9, 18)
    }

    fn now(&self) -> Timestamp {
        self.now.get()
    }

    fn zone(&self) -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }
}
