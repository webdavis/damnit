#![allow(dead_code)]

pub mod sandbox;

use std::time::{Duration, Instant};

use sandbox::Sandbox;

pub fn status_json(sb: &Sandbox, format_flag_and_more: &[&str]) -> serde_json::Value {
    let mut args = vec!["status"];
    args.extend_from_slice(format_flag_and_more);
    let (ok, out, err) = sb.dam(&args);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

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
