use jiff::civil::date;

use crate::config::Config;
use crate::ports::{HelperLauncher, Store};
use crate::service::{Dam, Remotes};
use crate::testing::{
    EchoCredentials, FakeEditRequests, FakeFilterFunctions, FakeHookEdits, FakeHookWarnings,
    FakeHooks, FakeInterruption, FakeProgress, FixedClock, FixedRandom,
};

pub struct Fixture {
    pub clock: FixedClock,
    pub random: FixedRandom,
    pub config: Config,
    pub filter_functions: FakeFilterFunctions,
    pub hooks: FakeHooks,
    pub hook_warnings: FakeHookWarnings,
    pub progress: FakeProgress,
    pub edit_requests: FakeEditRequests,
    pub hook_edits: FakeHookEdits,
    pub interruption: FakeInterruption,
}

impl Default for Fixture {
    fn default() -> Fixture {
        Fixture {
            clock: FixedClock(date(2026, 9, 18)),
            random: FixedRandom::new(0),
            config: Config::default(),
            filter_functions: FakeFilterFunctions::default(),
            hooks: FakeHooks::default(),
            hook_warnings: FakeHookWarnings::default(),
            progress: FakeProgress::default(),
            edit_requests: FakeEditRequests::default(),
            hook_edits: FakeHookEdits::default(),
            interruption: FakeInterruption::default(),
        }
    }
}

impl Fixture {
    pub fn dam<'a>(&'a self, store: &'a dyn Store) -> Dam<'a> {
        Dam {
            store,
            clock: &self.clock,
            random: &self.random,
            config: &self.config,
            access: &crate::access::Owner,
            filter_functions: &self.filter_functions,
            hooks: &self.hooks,
            hook_warnings: &self.hook_warnings,
            progress: &self.progress,
            edit_requests: &self.edit_requests,
            hook_edits: &self.hook_edits,
            interruption: &self.interruption,
            project_hooks: None,
        }
    }
}

pub fn remotes_launched_by(launcher: &dyn HelperLauncher) -> Remotes<'_> {
    Remotes {
        launcher,
        credentials: &EchoCredentials,
    }
}
