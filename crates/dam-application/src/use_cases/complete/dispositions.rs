use dam_domain::{
    Blockers, ChildDisposition, DependencyDisposition, Dispositions, Object, Oid, Task,
};

use super::load_task;
use crate::errors::UseCaseError;
use crate::ports::{ObjectRepository, Randomness};
use crate::use_cases::subtree::move_into;

pub(super) fn apply_dispositions(
    objects: &dyn ObjectRepository,
    random: &dyn Randomness,
    oid: &Oid,
    found: &Blockers,
    d: Dispositions,
) -> Result<(), UseCaseError> {
    let mut task = load_task(objects, oid)?;

    let target = match d.children {
        ChildDisposition::Keep => None,
        ChildDisposition::Up => Some(task.base.path.clone()),
        ChildDisposition::Into(_) if found.children.is_empty() => None,
        ChildDisposition::Into(name) => {
            let group_oid = Oid::generate(&mut |b| random.fill(b));
            let mut group = Task::new(group_oid, name.clone());
            group.base.path = task.base.path.clone();
            let folder = group
                .base
                .owned_folder()
                .ok_or_else(|| UseCaseError::Parse(format!("{name:?} cannot name a folder")))?;
            objects.put(&Object::Task(group))?;
            Some(folder)
        }
    };
    if let Some(folder) = target {
        for child in &found.children {
            move_into(objects, child, &folder)?;
        }
    }
    if d.dependencies == DependencyDisposition::Drop {
        task.base
            .depends
            .retain(|x| !found.dependencies.contains(x));
        objects.put(&Object::Task(task))?;
    }
    Ok(())
}
