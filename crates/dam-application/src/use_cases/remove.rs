use dam_domain::{Author, Item, ItemHash, Object, Oid, Permission};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::{contents, rebase_held};

#[derive(Debug, PartialEq, Eq)]
pub struct RemovePlan {
    pub oid: Oid,
    pub moved_up: Vec<Oid>,
    pub dependents: Vec<Oid>,
    pub attached: Vec<Oid>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Removal {
    Removed(RemovePlan),
    ItemDeclined(Box<Item>),
    Withdrawn(ItemAndTask),
    Waiting(ItemAndTask),
    Declined(ItemAndTask),
    Nothing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemAndTask {
    pub item: ItemHash,
    pub oid: Oid,
}

impl Dam<'_> {
    pub fn remove_or_send(&self, oid: &Oid) -> Result<Removal, UseCaseError> {
        let Author::Token(author) = self.access.author() else {
            return Ok(Removal::Removed(self.remove(oid)?));
        };
        let object = self.load_permitted(oid, Permission::remove)?;
        self.send_removal(author, object)
    }

    pub fn plan_remove(&self, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
        plan_for(self.store, &load(self.store, oid)?)
    }

    pub fn remove(&self, oid: &Oid) -> Result<RemovePlan, UseCaseError> {
        if self.access.token().is_some() {
            return Err(Refusal::NotOpenToTokens("rm").into());
        }
        atomically(self.store, || {
            let plan = carry_out(self.store, &load(self.store, oid)?)?;
            self.store.forget_hooked(oid)?;
            Ok(plan)
        })
    }
}

pub(crate) fn carry_out(
    objects: &dyn ObjectRepository,
    object: &Object,
) -> Result<RemovePlan, UseCaseError> {
    let oid = object.oid();
    let plan = plan_for(objects, object)?;
    if let Some(folder) = object.base().owned_folder() {
        rebase_held(objects, object, &folder, &object.base().path)?;
    }
    for child in &plan.moved_up {
        let mut moved = load(objects, child)?;
        moved.base_mut().parent = object.base().parent.clone();
        objects.put(&moved)?;
    }
    for dependent in &plan.dependents {
        if let Some(mut d) = objects.get(dependent)? {
            d.base_mut().depends.retain(|x| x != oid);
            objects.put(&d)?;
        }
    }
    objects.delete(oid)?;
    Ok(plan)
}

pub(crate) fn plan_for(
    objects: &dyn ObjectRepository,
    object: &Object,
) -> Result<RemovePlan, UseCaseError> {
    let oid = object.oid();
    let moved_up = contents(objects, object)?
        .iter()
        .map(|o| o.oid().clone())
        .collect();
    let all = objects.all()?;
    let dependents = objects.dependents_of(oid)?;
    let attached = all
        .iter()
        .filter(|o| o.as_task().is_some_and(|t| t.event.as_ref() == Some(oid)))
        .map(|o| o.oid().clone())
        .collect();
    Ok(RemovePlan {
        oid: oid.clone(),
        moved_up,
        dependents,
        attached,
    })
}

mod by_id;
mod by_token;
mod waiting;
mod withdraw;

#[cfg(test)]
mod tests;
