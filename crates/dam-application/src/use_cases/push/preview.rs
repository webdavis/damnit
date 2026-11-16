use std::collections::BTreeMap;

use dam_domain::{Change, Object, Oid, Permission};

use super::conflict_holds::hold_for_conflicts;
use super::hold::Hold;
use super::mutations::plan;
use super::owed::changes_to_send;
use super::skip::Skip;
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::RemoteName;
use crate::remote::{RemoteCapabilities, RemoteMutation};
use crate::service::Dam;
use crate::use_cases::as_it_stands::Wire;

pub struct PushPreview {
    pub remote: RemoteName,
    pub plan: Option<PlannedPush>,
}

pub struct PlannedPush {
    pub mutations: Vec<RemoteMutation>,
    pub held: Vec<Hold>,
    pub conflicted: Option<Vec<Oid>>,
    pub skipped: Vec<(Oid, Skip)>,
}

impl Dam<'_> {
    pub fn push_preview(
        &self,
        remote: Option<&str>,
        wire: Wire<'_>,
    ) -> Result<Vec<PushPreview>, UseCaseError> {
        let targets = self.remotes_permitting(Permission::RemotePush, remote)?;
        let mut previews = Vec::with_capacity(targets.len());
        for target in targets {
            previews.push(PushPreview {
                remote: target.name.clone(),
                plan: match self.recorded_capabilities(target)? {
                    Some(caps) => Some(self.as_seen_plan(self.planned(target, &caps, wire)?)?),
                    None => None,
                },
            });
        }
        Ok(previews)
    }

    fn recorded_capabilities(
        &self,
        remote: &RemoteConfig,
    ) -> Result<Option<RemoteCapabilities>, UseCaseError> {
        let kinds = self.store.accepted_kinds(&remote.name)?;
        let fields = self.store.accepted_fields(&remote.name)?;
        Ok(kinds.zip(fields).map(|(kinds, fields)| RemoteCapabilities {
            kinds,
            fields,
            ..RemoteCapabilities::default()
        }))
    }

    fn planned(
        &self,
        remote: &RemoteConfig,
        caps: &RemoteCapabilities,
        wire: Wire<'_>,
    ) -> Result<PlannedPush, UseCaseError> {
        let unpushed = self.store.unpushed(&remote.name)?;
        let record = self.store.attempts(&remote.name)?;
        let changes = changes_to_send(self.store, remote, &unpushed, &record)?;
        let owed = hold_for_conflicts(self.store, changes)?;
        let mut removed: BTreeMap<Oid, Object> = owed
            .changes
            .iter()
            .filter_map(|(oid, change)| match change {
                Change::Delete { before } => Some((oid.clone(), before.clone())),
                _ => None,
            })
            .collect();
        let mut planned = plan(self.store, wire, caps, remote, owed.changes, &record)?;
        for mutation in &mut planned.mutations {
            if mutation.object.is_none() {
                mutation.object = removed.remove(&mutation.oid);
            }
        }
        Ok(PlannedPush {
            mutations: planned.mutations,
            held: planned.unplaced,
            conflicted: Some(owed.held.into_iter().collect()),
            skipped: planned.skipped,
        })
    }
}

mod seen;
