use dam_domain::{Blockers, Object, Oid, Permission, Task};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::service::Dam;
use crate::use_cases::subtree::children;

pub struct CompletePlan {
    pub oid: Oid,
    pub blockers: Blockers,
}

impl Dam<'_> {
    pub fn plan_complete(&self, oid: &Oid) -> Result<CompletePlan, UseCaseError> {
        let task = match self.load_permitted(oid, |_| Permission::TaskComplete)? {
            Object::Task(t) => t,
            Object::Event(_) => return Err(Refusal::NotATask(oid.clone()).into()),
        };
        Ok(CompletePlan {
            oid: oid.clone(),
            blockers: self.blockers_of(&task)?,
        })
    }

    pub(crate) fn blockers_of(&self, task: &Task) -> Result<Blockers, UseCaseError> {
        let mut dependencies = Vec::new();
        for dep in &task.base.depends {
            if is_an_open_task(self.store, dep)? && !self.hides(dep)? {
                dependencies.push(dep.clone());
            }
        }
        let children: Vec<Oid> = children(self.store, &task.base)?
            .iter()
            .filter(|c| c.as_task().is_some_and(|t| !t.completion.is_done()))
            .filter(|c| self.access.sees(c))
            .map(|c| c.oid().clone())
            .collect();
        Ok(Blockers {
            dependencies,
            children,
        })
    }
}

fn is_an_open_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<bool, UseCaseError> {
    Ok(matches!(objects.get(oid)?, Some(Object::Task(t)) if !t.completion.is_done()))
}
