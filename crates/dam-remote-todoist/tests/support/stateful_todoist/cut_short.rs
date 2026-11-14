use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dam_helper::testing::Reply;
use serde_json::json;

use super::Held;

#[derive(Clone, Default)]
pub struct Release(Arc<AtomicBool>);

impl Release {
    pub fn now(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

pub(super) enum Cut {
    RateLimit,
    Hang(Release),
}

impl Cut {
    pub(super) fn answer(self) -> Option<Reply> {
        match self {
            Cut::RateLimit => {
                let limited = Reply::json(429, json!({"error": "Rate limit exceeded"}));
                Some(limited.with_header("Retry-After", "42"))
            }
            Cut::Hang(release) => {
                while !release.0.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(10));
                }
                None
            }
        }
    }
}

impl Held {
    pub fn rate_limiting_after(mut self, writes: u32) -> Held {
        self.cut_short_after = Some((writes, Cut::RateLimit));
        self
    }

    pub fn hanging_after(mut self, writes: u32) -> (Held, Release) {
        let release = Release::default();
        self.cut_short_after = Some((writes, Cut::Hang(release.clone())));
        (self, release)
    }

    pub(super) fn cuts_this_write_short(&mut self) -> Option<Cut> {
        match self.cut_short_after.take()? {
            (0, cut) => Some(cut),
            (left, cut) => {
                self.cut_short_after = Some((left - 1, cut));
                None
            }
        }
    }
}
