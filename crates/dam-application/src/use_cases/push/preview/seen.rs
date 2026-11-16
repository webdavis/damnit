use dam_domain::{Oid, Permission, Subject};

use super::PlannedPush;
use crate::errors::UseCaseError;
use crate::remote::RemoteMutation;
use crate::service::Dam;
use crate::use_cases::push::sight::HELD_BY_WHAT_YOU_CANNOT_SEE;

impl Dam<'_> {
    pub(super) fn as_seen_plan(&self, plan: PlannedPush) -> Result<PlannedPush, UseCaseError> {
        if self.access.token().is_none() {
            return Ok(plan);
        }
        let whole = |permission| self.access.permits(permission, &[Subject::Whole]);
        let mut mutations = Vec::new();
        for mutation in plan.mutations {
            let readable = match &mutation.object {
                Some(object) => self.access.sees(object),
                None => self.reads(&mutation.oid)?,
            };
            if readable {
                mutations.push(self.as_seen_mutation(mutation)?);
            }
        }
        let mut held = Vec::new();
        for mut hold in plan.held {
            if self.reads(&hold.oid)? {
                if self.any_hidden(&hold.waits_on)? {
                    hold.why = HELD_BY_WHAT_YOU_CANNOT_SEE.to_string();
                }
                held.push(hold);
            }
        }
        let conflicted = match plan.conflicted.filter(|_| whole(Permission::ConflictRead)) {
            Some(oids) => Some(self.read_only(oids)?),
            None => None,
        };
        Ok(PlannedPush {
            mutations,
            held,
            conflicted,
            skipped: self.read_rows(plan.skipped)?,
        })
    }

    fn as_seen_mutation(
        &self,
        mut mutation: RemoteMutation,
    ) -> Result<RemoteMutation, UseCaseError> {
        if let Some(object) = mutation.object.take() {
            let hidden_parent = match &object.base().parent {
                Some(parent) => self.hides(parent)?,
                None => false,
            };
            if hidden_parent {
                mutation.container_remote_id = None;
                mutation.container_oid = None;
            }
            mutation.object = Some(self.as_seen(object)?);
        }
        Ok(mutation)
    }

    fn read_rows<T>(&self, rows: Vec<(Oid, T)>) -> Result<Vec<(Oid, T)>, UseCaseError> {
        let mut seen = Vec::new();
        for (oid, row) in rows {
            if self.reads(&oid)? {
                seen.push((oid, row));
            }
        }
        Ok(seen)
    }

    fn read_only(&self, oids: Vec<Oid>) -> Result<Vec<Oid>, UseCaseError> {
        let mut seen = Vec::new();
        for oid in oids {
            if self.reads(&oid)? {
                seen.push(oid);
            }
        }
        Ok(seen)
    }
}
