//! The suite's shared parts: its own speed gate, and the sandbox that drives
//! the real binary. Each crate carries its own copy so the crate still builds
//! the day it moves to a repository of its own.

// Each test binary compiles this module and uses the part of it that binary
// needs, so the rest is unused there.
#![allow(dead_code)]

pub mod sandbox;

use std::time::{Duration, Instant};

/// What a test should finish inside. Over this it warns, which is the signal
/// to move it to a narrower level or shrink its fixture.
const BUDGET: Duration = Duration::from_secs(1);

/// What a test may never exceed. It sits above the budget rather than at it
/// because a host firewall evaluating the first outbound connection a fresh
/// test binary makes adds several seconds to whichever case runs first, and
/// that is a property of the machine rather than of the test. It is still far
/// below the thirty second http deadline a real regression would block on.
const CEILING: Duration = Duration::from_secs(10);

/// Measures the case it is bound in and reports at drop. Bind it to a name:
/// `let _guard = support::guard("name")`, not `let _ = ...`, which drops at
/// once and measures nothing.
#[must_use = "bind the guard to a name or it measures nothing"]
pub struct SpeedGuard {
    name: &'static str,
    started: Instant,
    allowed: Option<&'static str>,
}

/// Times one test case against the budget and the ceiling.
pub fn guard(name: &'static str) -> SpeedGuard {
    SpeedGuard {
        name,
        started: Instant::now(),
        allowed: None,
    }
}

impl SpeedGuard {
    /// Lets this one case run past the ceiling, naming the structural cause.
    pub fn allow_slow(mut self, reason: &'static str) -> SpeedGuard {
        self.allowed = Some(reason);
        self
    }
}

impl Drop for SpeedGuard {
    fn drop(&mut self) {
        let took = self.started.elapsed();
        if took > CEILING && self.allowed.is_none() && !std::thread::panicking() {
            panic!("{} took {took:?}, past the {CEILING:?} ceiling", self.name);
        }
        if took > BUDGET {
            let note = self.allowed.unwrap_or("no reason given");
            eprintln!("slow test: {} took {took:?} ({note})", self.name);
        }
    }
}
