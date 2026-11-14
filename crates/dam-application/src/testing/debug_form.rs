use dam_domain::carrying_only;

use crate::ports::{Declared, HelperError};
use crate::remote::RemoteMutation;

pub fn each_field_in_its_debug_form(mutation: &RemoteMutation) -> Result<Declared, HelperError> {
    Ok(mutation
        .fields
        .iter()
        .map(|field| {
            let value = mutation
                .object
                .as_ref()
                .map(|o| carrying_only(o, &[*field]));
            (*field, format!("{value:?}"))
        })
        .collect())
}
