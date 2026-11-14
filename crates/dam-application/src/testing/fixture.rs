use jiff::civil::date;

use crate::config::Config;
use crate::ports::{HelperLauncher, Store};
use crate::service::{Dam, Remotes};
use crate::testing::{EchoCredentials, FakeFilterFunctions, FixedClock, FixedRandom};

pub struct Fixture {
    pub clock: FixedClock,
    pub random: FixedRandom,
    pub config: Config,
    pub filter_functions: FakeFilterFunctions,
}

impl Default for Fixture {
    fn default() -> Fixture {
        Fixture {
            clock: FixedClock(date(2026, 9, 18)),
            random: FixedRandom::new(0),
            config: Config::default(),
            filter_functions: FakeFilterFunctions::default(),
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
        }
    }
}

pub fn remotes_launched_by(launcher: &dyn HelperLauncher) -> Remotes<'_> {
    Remotes {
        launcher,
        credentials: &EchoCredentials,
    }
}
