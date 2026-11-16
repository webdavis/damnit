use dam_domain::{Object, nearest_parents};

use crate::errors::UseCaseError;
use crate::ports::ObjectRepository;

pub(crate) fn put_adopting(
    objects: &dyn ObjectRepository,
    holder: &Object,
) -> Result<(), UseCaseError> {
    objects.put(holder)?;
    let Some(folder) = holder.base().owned_folder() else {
        return Ok(());
    };
    let all = objects.all()?;
    for mut held in all.iter().cloned() {
        let adopted = held.oid() != holder.oid()
            && held.base().parent == holder.base().parent
            && held.base().path.is_within(&folder)
            && nearest_parents(&held.base().path, &all) == [holder.oid().clone()];
        if adopted {
            held.base_mut().parent = Some(holder.oid().clone());
            objects.put(&held)?;
        }
    }
    Ok(())
}
