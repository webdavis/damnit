use std::cell::Cell;

use dam_domain::{Date, Object, Timestamp};
use jiff::SignedDuration;

use crate::config::HookEvent;
use crate::ports::{Clock, EditRequest, FunctionFailure, Histories, Hooks, Pull};
use crate::testing::FakeHooks;

pub(super) struct SlowHooks<'a> {
    pub(super) hooks: &'a FakeHooks,
    pub(super) now: Cell<Timestamp>,
    pub(super) each_call: SignedDuration,
}

impl<'a> SlowHooks<'a> {
    pub(super) fn taking(hooks: &'a FakeHooks, seconds: i64) -> SlowHooks<'a> {
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
        task: &Object,
        histories: &dyn Histories,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        self.hooks.run(event, task, histories)
    }

    fn pulled(
        &self,
        task: &Object,
        before: Option<&Object>,
        pull: &Pull,
    ) -> Result<Option<EditRequest>, FunctionFailure> {
        let later = self.now.get().checked_add(self.each_call).unwrap();
        self.now.set(later);
        self.hooks.pulled(task, before, pull)
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
