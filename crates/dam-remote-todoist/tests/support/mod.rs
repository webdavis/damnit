#![allow(dead_code)]

use std::time::{Duration, Instant};

const WARN_PAST: Duration = Duration::from_secs(1);

const FAIL_PAST: Duration = Duration::from_secs(10);

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
        if took > FAIL_PAST && self.allowed.is_none() && !std::thread::panicking() {
            panic!(
                "{} took {took:?}, past the {FAIL_PAST:?} ceiling",
                self.name
            );
        }
        if took > WARN_PAST {
            let note = self.allowed.unwrap_or("no reason given");
            eprintln!("slow test: {} took {took:?} ({note})", self.name);
        }
    }
}
