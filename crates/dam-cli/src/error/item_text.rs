use std::fmt;

use dam_application::ItemRefusal;

use super::refusal_text::open_blockers;

pub(super) struct ItemText<'a>(pub(super) &'a ItemRefusal);

impl fmt::Display for ItemText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ItemRefusal::NothingChanged => f.write_str("that leaves the object as it is"),
            ItemRefusal::Declined(hash) => write!(
                f,
                "the owner declined this as {}; it is not offered again",
                hash.short()
            ),
            ItemRefusal::NotAnObject(hash) => write!(
                f,
                "{} is a waiting item, not an object; dam add or dam rm it",
                hash.short()
            ),
            ItemRefusal::NotYours(hash) => write!(
                f,
                "{} was sent by another token; a token may remove only its own items",
                hash.short()
            ),
            ItemRefusal::NotWaiting { hash, outcome } => {
                write!(f, "{} was already {outcome}", hash.short())
            }
            ItemRefusal::TargetGone { hash, target } => write!(
                f,
                "{} changes {}, which is no longer in the working layer",
                hash.short(),
                target.short()
            ),
            ItemRefusal::TargetNotCommitted { hash, target } => write!(
                f,
                "{} changes {}, which has never been committed; commit it, then add {}",
                hash.short(),
                target.short(),
                hash.short()
            ),
            ItemRefusal::TargetStaged { hash, target } => write!(
                f,
                "{} changes {}, which already has a staged change; commit or reset it, then add {}",
                hash.short(),
                target.short(),
                hash.short()
            ),
            ItemRefusal::Clash {
                hash,
                target,
                fields,
            } => write!(
                f,
                "{} changes {} of {}, which you changed too; run dam resolve {} --ours|--theirs",
                hash.short(),
                fields
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                target.short(),
                hash.short()
            ),
            ItemRefusal::RemovesAChangedTask {
                hash,
                target,
                fields,
            } => write!(
                f,
                "{} removes {}, whose {} you changed; run dam resolve {} --ours|--theirs",
                hash.short(),
                target.short(),
                fields
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                hash.short()
            ),
            ItemRefusal::Cycle { hash, target, path } => write!(
                f,
                "{} makes {} depend on what leads back to it, a cycle through {}; decline it with dam rm {}",
                hash.short(),
                target.short(),
                path.iter()
                    .map(|o| o.short())
                    .collect::<Vec<_>>()
                    .join(" -> "),
                hash.short()
            ),
            ItemRefusal::Blocked {
                hash,
                target,
                blockers,
            } => write!(
                f,
                "{} completes {}, which has {}; finish those first, or run dam done {} --force and dam rm {}",
                hash.short(),
                target.short(),
                open_blockers(blockers),
                target.short(),
                hash.short()
            ),
            ItemRefusal::EditedSinceAdded { hash, target } => write!(
                f,
                "{} is new from {}, and you have changed it since you added it; commit it first",
                target.short(),
                hash.short()
            ),
            ItemRefusal::SubjectNamesNoFolder { hash, target } => write!(
                f,
                "{} renames {}, whose folder holds objects, to a subject that cannot name a folder; decline it with dam rm {}",
                hash.short(),
                target.short(),
                hash.short()
            ),
        }
    }
}
