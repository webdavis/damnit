use crate::{Change, Field, Object, touched_fields};

pub fn same_proposal(a: &Change, b: &Change) -> bool {
    match (a, b) {
        (Change::Create { after: x }, Change::Create { after: y }) => {
            let mut x = x.clone();
            x.base_mut().oid = y.oid().clone();
            x == *y
        }
        (Change::Update { after: x, .. }, Change::Update { after: y, .. }) => {
            let fields = touched_fields(a);
            x.oid() == y.oid()
                && fields == touched_fields(b)
                && fields.iter().all(|f| agree(*f, x, y))
        }
        (Change::Delete { before: x }, Change::Delete { before: y }) => x.oid() == y.oid(),
        _ => false,
    }
}

fn agree(field: Field, a: &Object, b: &Object) -> bool {
    match (field, a.as_task(), b.as_task()) {
        (Field::Done, Some(x), Some(y)) => x.completion.is_done() == y.completion.is_done(),
        _ => field.same(a, b),
    }
}
