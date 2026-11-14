use dam_application::RemovePlan;
use dam_domain::{Item, ItemHash, Oid, Timestamp};
use serde::Serialize;

use super::rows::{ChangeRow, visible};
use crate::commands::follow_ons::FollowOns;
use crate::context::Context;
use crate::view::{Line, display};

#[derive(Serialize)]
pub(super) struct ItemRow<'a> {
    #[serde(serialize_with = "display")]
    hash: &'a ItemHash,
    item: &'static str,
    #[serde(skip)]
    target: &'a Oid,
    #[serde(flatten)]
    change: ChangeRow<'a>,
    #[serde(serialize_with = "display")]
    created: &'a Timestamp,
    age: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    removes: Option<FollowOns<'a>>,
}

impl<'a> ItemRow<'a> {
    pub(super) fn new(item: &'a Item, now: Timestamp, full: bool) -> ItemRow<'a> {
        ItemRow {
            hash: &item.hash,
            item: item.kind().as_str(),
            target: item.target(),
            change: ChangeRow::new(&item.change, Some(&item.author), full),
            created: &item.created,
            age: now.duration_since(item.created).as_secs().max(0),
            removes: None,
        }
    }

    pub(super) fn removing(self, plan: Option<&'a RemovePlan>) -> ItemRow<'a> {
        ItemRow {
            removes: plan.map(FollowOns::of),
            ..self
        }
    }

    pub(super) fn told(self, ctx: &'a Context, item: &'a Item) -> ItemRow<'a> {
        let details = visible(
            ctx.access.as_ref(),
            Some(&item.author),
            item.details.as_ref(),
        );
        ItemRow {
            change: self.change.sent(ctx.config.owner_name(), details),
            ..self
        }
    }

    pub(super) fn lines(&self, owner: &str) -> Vec<Line> {
        let target = self.target;
        let id = if target == self.hash.oid() {
            self.hash.short().to_string()
        } else {
            format!("{}  {}", self.hash.short(), target.short())
        };
        let mut lines = self.change.lines("  ", &id, owner, &[age(self.age)]);
        if let Some(counts) = self.removes.as_ref().and_then(FollowOns::counts) {
            lines.push(Line::Whole(format!("    {counts}")));
        }
        lines
    }
}

fn age(seconds: i64) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    match seconds {
        s if s < MINUTE => format!("{s}s"),
        s if s < HOUR => format!("{}m", s / MINUTE),
        s if s < DAY => format!("{}h", s / HOUR),
        s => format!("{}d", s / DAY),
    }
}

#[cfg(test)]
mod tests;
