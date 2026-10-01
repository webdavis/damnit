#![allow(dead_code)]

use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_secs(1);

const CEILING_CLEARING_A_FIREWALLS_FIRST_CONNECTION_DELAY: Duration = Duration::from_secs(10);

#[must_use = "bind the guard to a name or it measures nothing"]
pub struct SpeedGuard {
    name: &'static str,
    started: Instant,
    allowed: Option<&'static str>,
}

pub fn guard(name: &'static str) -> SpeedGuard {
    SpeedGuard {
        name,
        started: Instant::now(),
        allowed: None,
    }
}

impl SpeedGuard {
    pub fn allow_slow(mut self, structural_cause: &'static str) -> SpeedGuard {
        self.allowed = Some(structural_cause);
        self
    }
}

impl Drop for SpeedGuard {
    fn drop(&mut self) {
        let took = self.started.elapsed();
        let ceiling = CEILING_CLEARING_A_FIREWALLS_FIRST_CONNECTION_DELAY;
        if took > ceiling && self.allowed.is_none() && !std::thread::panicking() {
            panic!("{} took {took:?}, past the {ceiling:?} ceiling", self.name);
        }
        if took > BUDGET {
            let note = self.allowed.unwrap_or("no reason given");
            eprintln!(
                "slow test: {} took {took:?} ({note}); move it to a narrower level or shrink its fixture",
                self.name
            );
        }
    }
}

pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .into()
}
