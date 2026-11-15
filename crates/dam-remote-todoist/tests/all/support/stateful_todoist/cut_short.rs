use dam_helper::testing::Reply;
use serde_json::json;

use super::Held;

pub(super) enum Cut {
    RateLimit,
    Unauthorized,
}

impl Cut {
    pub(super) fn answer(self) -> Option<Reply> {
        match self {
            Cut::RateLimit => {
                let limited = Reply::json(429, json!({"error": "Rate limit exceeded"}));
                Some(limited.with_header("Retry-After", "42"))
            }
            Cut::Unauthorized => Some(Reply::json(401, json!({"error": "Unauthorized"}))),
        }
    }
}

impl Held {
    pub fn rate_limiting_after(mut self, writes: u32) -> Held {
        self.cut_short_after = Some((writes, Cut::RateLimit));
        self
    }

    pub fn rate_limit_after(&mut self, writes: u32) {
        self.cut_short_after = Some((writes, Cut::RateLimit));
    }

    pub fn unauthorized_after(mut self, writes: u32) -> Held {
        self.cut_short_after = Some((writes, Cut::Unauthorized));
        self
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
