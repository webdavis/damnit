use dam_domain::{Object, Oid, Path, Permission, Task};

use super::NewTask;
use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::items::land::Landed;

impl Dam<'_> {
    pub fn hold_project(&self, place: &Path) -> Result<Option<Landed>, UseCaseError> {
        let Some(parent) = place.parent() else {
            return Ok(None);
        };
        let subject = place.as_str().trim_end_matches('/');
        let subject = subject.rsplit('/').next().unwrap_or(subject).to_string();
        let mut probe = Task::new(Oid::generate(&mut |b| b.fill(0)), subject.clone());
        probe.base.path = parent.clone();
        self.refuse_unless_permitted(Permission::TaskCreate, None, &Object::Task(probe))?;
        let held = self
            .store
            .all()?
            .iter()
            .filter(|o| self.access.sees(o))
            .any(|o| o.base().owned_folder().as_ref() == Some(place));
        if held {
            return Ok(None);
        }
        self.new_task(NewTask {
            parent: None,
            subject,
            path: parent,
            priority: Default::default(),
            due: None,
            deadline: None,
            labels: Default::default(),
            body: String::new(),
            id: None,
        })
        .map(Some)
    }
}
