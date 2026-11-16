use jiff::{Timestamp, Zoned};

use super::moment::Moment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowEdge(Moment);

impl WindowEdge {
    pub fn read(text: &str) -> Option<WindowEdge> {
        Moment::read(text).map(WindowEdge)
    }

    pub fn opens(&self, now: &Zoned) -> Option<Timestamp> {
        self.0.resolve(now).map(|s| s.from)
    }

    pub fn closes(&self, now: &Zoned) -> Option<Timestamp> {
        self.0.resolve(now).map(|s| s.end())
    }

    pub fn shapes() -> &'static [&'static str] {
        &[
            "2026-11-20",
            "2026-11-20T09:00",
            "2026-W47",
            "2026-11",
            "2026",
            "today",
            "today+3",
            "week",
            "week+1",
            "month-1",
            "year",
            "now",
            "now+2h",
        ]
    }
}
