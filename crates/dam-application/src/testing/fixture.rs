use jiff::civil::date;

use crate::config::Config;
use crate::ports::{HelperLauncher, Store};
use crate::service::{Dam, Remotes};
use crate::testing::{EchoCredentials, FixedClock, FixedRandom};

pub struct Fixture {
    pub clock: FixedClock,
    pub random: FixedRandom,
    pub config: Config,
}

impl Default for Fixture {
    fn default() -> Fixture {
        Fixture {
            clock: FixedClock(date(2026, 9, 18)),
            random: FixedRandom::new(0),
            config: Config::default(),
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
        }
    }
}

pub fn remotes_launched_by(launcher: &dyn HelperLauncher) -> Remotes<'_> {
    Remotes {
        launcher,
        credentials: &EchoCredentials,
    }
}
