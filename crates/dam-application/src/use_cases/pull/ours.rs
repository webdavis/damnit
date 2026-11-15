use dam_domain::Object;

use crate::errors::UseCaseError;
use crate::ports::{Attempt, RemoteHelper};
use crate::remote::RemoteCapabilities;
use crate::use_cases::as_it_stands::as_it_stands;

pub(super) fn keep_what_we_sent(
    helper: &dyn RemoteHelper,
    caps: &RemoteCapabilities,
    local: &Object,
    mut theirs: Object,
    attempts: &[Attempt],
) -> Result<Object, UseCaseError> {
    if attempts.is_empty() {
        return Ok(theirs);
    }
    let held = as_it_stands(helper, caps, &theirs)?;
    let here = as_it_stands(helper, caps, local)?;
    for field in &caps.fields {
        let Some(value) = held.get(field) else {
            continue;
        };
        let sent_by_dam = attempts
            .iter()
            .any(|a| a.declared.get(field) == Some(value));
        if here.get(field) != Some(value) && sent_by_dam {
            field.copy(local, &mut theirs);
        }
    }
    Ok(theirs)
}
