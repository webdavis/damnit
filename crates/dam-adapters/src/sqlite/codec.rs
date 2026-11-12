use dam_application::StoreError;
use dam_domain::{Change, Object, Op};

pub(super) fn object_to_json(o: &Object) -> Result<String, StoreError> {
    serde_json::to_string(&crate::wire::to_wire(o, None))
        .map_err(|e| StoreError::Failed(format!("stored object: {e}")))
}

pub(super) fn object_from_json(s: &str) -> Result<Object, StoreError> {
    let w: dam_protocol::WireObject =
        serde_json::from_str(s).map_err(|e| StoreError::Failed(format!("stored object: {e}")))?;
    crate::wire::from_wire(&w).map_err(|e| StoreError::Failed(format!("stored object: {e}")))
}

pub(super) fn op_from_text(s: &str) -> Result<Op, StoreError> {
    Op::parse(s).ok_or_else(|| StoreError::Failed(format!("stored op {s:?}")))
}

pub(super) fn change_from_row(
    op: &str,
    before: Option<String>,
    after: Option<String>,
) -> Result<Change, StoreError> {
    let op = op_from_text(op)?;
    let before = before.map(|j| object_from_json(&j)).transpose()?;
    let after = after.map(|j| object_from_json(&j)).transpose()?;
    Change::between(before, after)
        .filter(|change| change.op() == op)
        .ok_or_else(|| {
            StoreError::Failed(format!(
                "stored {} change disagrees with its before and after",
                op.as_str()
            ))
        })
}

mod notices;

pub(super) use notices::{notice_from_json, notice_to_json};

#[cfg(test)]
mod tests;
