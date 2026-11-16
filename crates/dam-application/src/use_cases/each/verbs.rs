use dam_domain::{Author, Change, Force, Oid, Path, Permission, Tracked};

use super::{Selector, SetChanges, SetRun, sent};
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::edit::EditFields;
use crate::use_cases::load::load;
use crate::use_cases::remove::carry_out;
use crate::use_cases::subtree::is_held_by;

impl Dam<'_> {
    pub fn edit_each(
        &self,
        selector: &Selector,
        fields: EditFields,
        run: SetRun,
    ) -> Result<SetChanges, UseCaseError> {
        self.each(selector, run, false, |oid| {
            Ok(sent(self.edit_in_place(oid, fields.clone())?))
        })
    }

    pub fn complete_each(
        &self,
        selector: &Selector,
        force: &dyn Fn(&Oid) -> Force,
        tracked: &dyn Fn(&Oid) -> Tracked,
        run: SetRun,
    ) -> Result<SetChanges, UseCaseError> {
        self.each(selector, run, true, |oid| {
            let (_, landed) = self.complete_in_place(oid, force(oid), tracked(oid))?;
            Ok(sent(landed))
        })
    }

    pub fn relocate_each(
        &self,
        selector: &Selector,
        to: Option<&Path>,
        parent: Option<&Oid>,
        run: SetRun,
    ) -> Result<SetChanges, UseCaseError> {
        if self.access.token().is_some() {
            return Err(Refusal::NotOpenToTokens("mv").into());
        }
        let set = self.select(selector)?;
        self.each(selector, run, false, |oid| {
            for other in set.iter().filter(|other| *other != oid) {
                if is_held_by(self.store, oid, other)? {
                    return Ok(None);
                }
            }
            self.relocate_in_place(oid, to, parent)?;
            Ok(None)
        })
    }

    pub fn remove_each(
        &self,
        selector: &Selector,
        run: SetRun,
    ) -> Result<SetChanges, UseCaseError> {
        self.each(selector, run, false, |oid| {
            let object = self.load_permitted(oid, Permission::remove)?;
            match self.access.author() {
                Author::Token(author) => {
                    let removal = Change::Delete { before: object };
                    Ok(Some(self.send(author, removal, &|_| true)?))
                }
                _ => {
                    carry_out(self.store, &load(self.store, oid)?)?;
                    Ok(None)
                }
            }
        })
    }
}
