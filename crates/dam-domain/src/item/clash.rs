use super::proposal::agree;
use crate::{Field, Object, changed_fields};

pub fn clashing_fields(
    before: &Object,
    after: &Object,
    committed: &Object,
    working: &Object,
) -> Vec<Field> {
    changed_fields(before, after)
        .into_iter()
        .filter(|f| owner_moved(*f, before, committed, working) && !agree(*f, working, after))
        .collect()
}

pub fn fields_the_owner_moved(before: &Object, committed: &Object, working: &Object) -> Vec<Field> {
    Field::ALL
        .into_iter()
        .filter(|f| owner_moved(*f, before, committed, working))
        .collect()
}

fn owner_moved(field: Field, before: &Object, committed: &Object, working: &Object) -> bool {
    !agree(field, working, committed) || !agree(field, working, before)
}
