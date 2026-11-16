use dam_domain::{Object, Oid, Path};

use crate::errors::UseCaseError;
use crate::ports::{Conflict, Notice};
use crate::service::Dam;

const FAILED_FOR_A_REASON_KEPT_FROM_YOU: &str =
    "did not go through; the owner sees why in dam status";

impl Dam<'_> {
    pub(super) fn seen_conflicts(&self) -> Result<(Vec<Conflict>, bool), UseCaseError> {
        let conflicts = self.store.conflicts()?;
        if self.access.token().is_none() {
            return Ok((conflicts, false));
        }
        let all = conflicts.len();
        let mut seen = Vec::new();
        for conflict in conflicts {
            let versions_seen = conflict
                .ours
                .iter()
                .chain([&conflict.theirs])
                .all(|version| self.access.sees(version));
            if versions_seen && self.reads(&conflict.oid)? {
                seen.push(conflict);
            }
        }
        let hidden = seen.len() < all;
        Ok((seen, hidden))
    }

    pub(super) fn seen_notices(&self) -> Result<(Vec<Notice>, bool), UseCaseError> {
        let notices = self.store.notices()?;
        if self.access.token().is_none() {
            return Ok((notices, false));
        }
        let mut seen = Vec::new();
        let mut hidden_failure = false;
        for notice in notices {
            let failure = matches!(
                notice,
                Notice::PushFailed { .. } | Notice::UnansweredCreateDropped { .. }
            );
            match self.as_seen_notice(notice)? {
                Some(notice) => seen.push(notice),
                None => hidden_failure |= failure,
            }
        }
        Ok((seen, hidden_failure))
    }

    fn as_seen_notice(&self, notice: Notice) -> Result<Option<Notice>, UseCaseError> {
        Ok(match notice {
            Notice::RemovedUpstream { ref oid, .. } | Notice::KindChanged { ref oid, .. } => {
                self.reads(oid)?.then_some(notice)
            }
            Notice::UnansweredCreateDropped {
                ref oid, ref path, ..
            } => (self.reads(oid)? && self.still_at(oid, path)?).then_some(notice),
            Notice::PushFailed { remote, oid, why } => {
                let why = match self.reads_everything() {
                    true => why,
                    false => FAILED_FOR_A_REASON_KEPT_FROM_YOU.to_string(),
                };
                self.reads(&oid)?
                    .then_some(Notice::PushFailed { remote, oid, why })
            }
            Notice::EventCancelled { oid, subject, .. } => match self.reads(&oid)? {
                true => Some(Notice::EventCancelled {
                    attached: self.attached_seen(&oid)?,
                    oid,
                    subject,
                }),
                false => None,
            },
            Notice::MissingUpstream { remote, tasks } => {
                let mut seen = Vec::new();
                for task in tasks {
                    if self.reads(&task.oid)? {
                        seen.push(task);
                    }
                }
                (!seen.is_empty()).then_some(Notice::MissingUpstream {
                    remote,
                    tasks: seen,
                })
            }
            Notice::PullSkipped { .. } | Notice::TaskPulledFailed { .. } => None,
        })
    }

    fn still_at(&self, oid: &Oid, noted: &Path) -> Result<bool, UseCaseError> {
        let at = |object: Option<Object>| object.is_none_or(|o| &o.base().path == noted);
        Ok(self.reads_everything()
            || (at(self.store.get(oid)?) && at(self.store.committed(oid)?)))
    }

    fn attached_seen(&self, event: &Oid) -> Result<usize, UseCaseError> {
        Ok(self
            .store
            .all()?
            .iter()
            .filter(|o| self.access.sees(o))
            .filter(|o| matches!(o, Object::Task(t) if t.event.as_ref() == Some(event)))
            .count())
    }
}
