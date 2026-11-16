use dam_application::SyncReport;
use serde::Serialize;

use super::pull_view::PullRow;
use super::view::PushRow;
use crate::style::heading;
use crate::view::{Human, View};

#[derive(Serialize)]
pub(super) struct Synced<'a> {
    remote: &'a str,
    pulled: PullRow<'a>,
    pushed: PushRow<'a>,
}

impl<'a> Synced<'a> {
    pub(super) fn new(r: &'a SyncReport) -> Synced<'a> {
        Synced {
            remote: &r.pulled.remote.0,
            pulled: PullRow::new(&r.pulled),
            pushed: PushRow::new(&r.pushed),
        }
    }
}

impl View for Synced<'_> {
    fn human(&self) -> Human {
        Human::styled(|paint| {
            let blurb = format!("what {} sent, then what it took", self.remote);
            let mut lines = vec![heading(paint, "sync", &blurb)];
            lines.extend(self.pulled.lines(paint, "pulled"));
            for line in self.pushed.lines(paint, "pushed") {
                if !lines.contains(&line) {
                    lines.push(line);
                }
            }
            lines
        })
    }
}
