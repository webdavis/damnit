use dam_application::{Notice, RemoteName, StoreError};
use dam_domain::{Change, Kind, Object, Oid, Op};
use rusqlite::Params;
use serde::{Deserialize, Serialize};

use super::objects::sql;

pub(super) fn object_to_json(o: &Object) -> Result<String, StoreError> {
    serde_json::to_string(&crate::wire::to_wire(o, None))
        .map_err(|e| StoreError::Failed(format!("stored object: {e}")))
}

pub(super) fn object_from_json(s: &str) -> Result<Object, StoreError> {
    let w: dam_protocol::WireObject =
        serde_json::from_str(s).map_err(|e| StoreError::Failed(format!("stored object: {e}")))?;
    crate::wire::from_wire(&w).map_err(|e| StoreError::Failed(format!("stored object: {e}")))
}

pub(super) fn op_to_text(op: Op) -> &'static str {
    match op {
        Op::Create => "create",
        Op::Update => "update",
        Op::Delete => "delete",
    }
}

pub(super) fn op_from_text(s: &str) -> Result<Op, StoreError> {
    match s {
        "create" => Ok(Op::Create),
        "update" => Ok(Op::Update),
        "delete" => Ok(Op::Delete),
        other => Err(StoreError::Failed(format!("stored op {other:?}"))),
    }
}

fn change_from_row(
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
                op_to_text(op)
            ))
        })
}

pub(super) fn read_changes(
    stmt: &mut rusqlite::Statement,
    params: impl Params,
) -> Result<Vec<Change>, StoreError> {
    stmt.query_map(params, |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    })
    .map_err(sql)?
    .map(|r| {
        r.map_err(sql)
            .and_then(|(op, b, a)| change_from_row(&op, b, a))
    })
    .collect()
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum NoticeRow {
    RemovedUpstream {
        remote: String,
        oid: String,
        subject: String,
    },
    EventCancelled {
        oid: String,
        subject: String,
        attached: usize,
    },
    PushFailed {
        remote: String,
        oid: String,
        why: String,
    },
    PullFailed {
        remote: String,
        why: String,
    },
    KindChanged {
        oid: String,
        ours: String,
        theirs: String,
    },
}

pub(super) fn notice_to_json(n: &Notice) -> Result<String, StoreError> {
    let row = match n {
        Notice::RemovedUpstream {
            remote,
            oid,
            subject,
        } => NoticeRow::RemovedUpstream {
            remote: remote.0.clone(),
            oid: oid.to_string(),
            subject: subject.clone(),
        },
        Notice::EventCancelled {
            oid,
            subject,
            attached,
        } => NoticeRow::EventCancelled {
            oid: oid.to_string(),
            subject: subject.clone(),
            attached: *attached,
        },
        Notice::PushFailed { remote, oid, why } => NoticeRow::PushFailed {
            remote: remote.0.clone(),
            oid: oid.to_string(),
            why: why.clone(),
        },
        Notice::PullFailed { remote, why } => NoticeRow::PullFailed {
            remote: remote.0.clone(),
            why: why.clone(),
        },
        Notice::KindChanged { oid, ours, theirs } => NoticeRow::KindChanged {
            oid: oid.to_string(),
            ours: ours.as_str().to_string(),
            theirs: theirs.as_str().to_string(),
        },
    };
    serde_json::to_string(&row).map_err(|e| StoreError::Failed(format!("stored notice: {e}")))
}

pub(super) fn notice_from_json(s: &str) -> Result<Notice, StoreError> {
    let row: NoticeRow =
        serde_json::from_str(s).map_err(|e| StoreError::Failed(format!("stored notice: {e}")))?;
    let oid =
        |t: &str| Oid::parse(t).map_err(|e| StoreError::Failed(format!("stored notice: {e}")));
    let kind = |t: &str| {
        Kind::parse(t)
            .ok_or_else(|| StoreError::Failed(format!("stored notice: unknown kind {t:?}")))
    };
    Ok(match row {
        NoticeRow::RemovedUpstream {
            remote,
            oid: o,
            subject,
        } => Notice::RemovedUpstream {
            remote: RemoteName(remote),
            oid: oid(&o)?,
            subject,
        },
        NoticeRow::EventCancelled {
            oid: o,
            subject,
            attached,
        } => Notice::EventCancelled {
            oid: oid(&o)?,
            subject,
            attached,
        },
        NoticeRow::PushFailed {
            remote,
            oid: o,
            why,
        } => Notice::PushFailed {
            remote: RemoteName(remote),
            oid: oid(&o)?,
            why,
        },
        NoticeRow::PullFailed { remote, why } => Notice::PullFailed {
            remote: RemoteName(remote),
            why,
        },
        NoticeRow::KindChanged {
            oid: o,
            ours,
            theirs,
        } => Notice::KindChanged {
            oid: oid(&o)?,
            ours: kind(&ours)?,
            theirs: kind(&theirs)?,
        },
    })
}

#[cfg(test)]
mod tests;
