pub mod answers;
pub mod background_sync;
pub mod commands;
pub mod config_files;
pub mod config_tokens;
pub mod doctor;
pub mod errors;
pub mod fixture;
pub mod golden;
pub mod help_flags;
pub mod helper_scripts;
pub mod hidden;
pub mod init_answer;
pub mod own;
pub mod page_examples;
pub mod request;
pub mod sandbox;
pub mod schema;
pub mod small_config;
pub mod token_table;
pub mod tokens;

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

fn fails_the_test(
    took: Duration,
    allowed: Option<&str>,
    run_alone_on_an_idle_machine: bool,
) -> bool {
    run_alone_on_an_idle_machine
        && allowed.is_none()
        && took > CEILING_CLEARING_A_FIREWALLS_FIRST_CONNECTION_DELAY
}

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
        let alone = std::env::var_os("DAM_TESTS_RUN_ALONE_ON_AN_IDLE_MACHINE").is_some();
        if fails_the_test(took, self.allowed, alone) && !std::thread::panicking() {
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
