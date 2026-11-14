use dam_helper::testing::Reply;
use serde_json::json;

use super::Held;

impl Held {
    pub fn rate_limiting_after(mut self, writes: u32) -> Held {
        self.writes_before_a_rate_limit = Some(writes);
        self
    }

    pub(super) fn rate_limits_this_write(&mut self) -> Option<Reply> {
        match self.writes_before_a_rate_limit.take()? {
            0 => {
                let limited = Reply::json(429, json!({"error": "Rate limit exceeded"}));
                Some(limited.with_header("Retry-After", "42"))
            }
            left => {
                self.writes_before_a_rate_limit = Some(left - 1);
                None
            }
        }
    }
}
