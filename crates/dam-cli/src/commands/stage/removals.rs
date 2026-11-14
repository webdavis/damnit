use dam_application::AddedRemoval;
use dam_domain::{ItemHash, Oid};
use serde::Serialize;

use crate::commands::follow_ons::FollowOns;
use crate::view::display;

#[derive(Serialize)]
pub(super) struct AddedRemovals<'a> {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    removed: Vec<RemovedRow<'a>>,
}

#[derive(Serialize)]
struct RemovedRow<'a> {
    #[serde(serialize_with = "display")]
    item: &'a ItemHash,
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    #[serde(flatten)]
    follow_ons: FollowOns<'a>,
}

impl<'a> AddedRemovals<'a> {
    pub(super) fn new(removals: &'a [AddedRemoval]) -> AddedRemovals<'a> {
        let mut rows = AddedRemovals {
            removed: Vec::new(),
        };
        for removal in removals {
            match removal {
                AddedRemoval::Removed { item, plan } => rows.removed.push(RemovedRow {
                    item,
                    oid: &plan.oid,
                    follow_ons: FollowOns::of(plan),
                }),
            }
        }
        rows
    }

    pub(super) fn lines(&self) -> impl Iterator<Item = String> + '_ {
        self.removed.iter().flat_map(|r| {
            let lines = r.follow_ons.lines().into_iter();
            std::iter::once(format!("{} removed {}", r.item.short(), r.oid.short()))
                .chain(lines.map(|line| format!("  {line}")))
        })
    }
}
