use dam_domain::{Field, Oid, Path};

use super::Removals;
use crate::remote::{MutationOp, RemoteMutation};

pub(super) enum Unreleased<'a> {
    MovedWithoutTellingUpstream(&'a Oid),
    LeftUnplaced(&'a Oid),
}

pub(super) fn places(mutation: &RemoteMutation) -> bool {
    mutation.op == MutationOp::Create || mutation.fields.contains(&Field::Path)
}

impl Removals {
    pub(super) fn unreleased<'a>(&self, stuck: &'a Oid, folder: &Path) -> Option<Unreleased<'a>> {
        let held = &self.upstream.get(stuck)?.path;
        if self.local.get(stuck) != Some(held) {
            return Some(Unreleased::MovedWithoutTellingUpstream(stuck));
        }
        if !self.placed_by_this_push(stuck, held, folder) {
            return Some(Unreleased::LeftUnplaced(stuck));
        }
        None
    }

    fn placed_by_this_push(&self, stuck: &Oid, held: &Path, folder: &Path) -> bool {
        if held == folder {
            return self.placed.contains(stuck);
        }
        self.upstream.iter().any(|(oid, base)| {
            &base.path == folder
                && self.placed.contains(oid)
                && base.owned_folder().is_some_and(|own| held.is_within(&own))
        })
    }
}
