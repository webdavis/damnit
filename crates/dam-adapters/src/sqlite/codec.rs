use dam_application::{Notice, RemoteName, StoreError, wire};
use dam_domain::{Change, Object, Oid, Op};
use rusqlite::Params;
use serde::{Deserialize, Serialize};

use super::objects::sql;

pub(super) fn object_to_json(o: &Object) -> Result<String, StoreError> {
    serde_json::to_string(&wire::to_wire(o, None))
        .map_err(|e| StoreError(format!("stored object: {e}")))
}

pub(super) fn object_from_json(s: &str) -> Result<Object, StoreError> {
    let w: dam_protocol::WireObject =
        serde_json::from_str(s).map_err(|e| StoreError(format!("stored object: {e}")))?;
    wire::from_wire(&w).map_err(|e| StoreError(format!("stored object: {e}")))
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
        other => Err(StoreError(format!("stored op {other:?}"))),
    }
}

/// Rebuilds a `Change` from a `stage` or `commit_changes` row's columns.
pub(super) fn change_from_row(
    oid: &str,
    op: &str,
    before: Option<String>,
    after: Option<String>,
) -> Result<Change, StoreError> {
    Ok(Change {
        oid: Oid::parse(oid).map_err(|e| StoreError(e.to_string()))?,
        op: op_from_text(op)?,
        before: before.map(|j| object_from_json(&j)).transpose()?,
        after: after.map(|j| object_from_json(&j)).transpose()?,
    })
}

/// Runs a prepared `oid, op, before_json, after_json` query and decodes every row.
/// Shared by `stage`'s `staged()` and `commits`' `commit_changes()`, whose only
/// difference is the SQL and its bind parameters.
pub(super) fn read_changes(
    stmt: &mut rusqlite::Statement,
    params: impl Params,
) -> Result<Vec<Change>, StoreError> {
    stmt.query_map(params, |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })
    .map_err(sql)?
    .map(|r| {
        r.map_err(sql)
            .and_then(|(o, op, b, a)| change_from_row(&o, &op, b, a))
    })
    .collect()
}

#[allow(dead_code)]
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
}

#[allow(dead_code)]
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
    };
    serde_json::to_string(&row).map_err(|e| StoreError(format!("stored notice: {e}")))
}

#[allow(dead_code)]
pub(super) fn notice_from_json(s: &str) -> Result<Notice, StoreError> {
    let row: NoticeRow =
        serde_json::from_str(s).map_err(|e| StoreError(format!("stored notice: {e}")))?;
    let oid = |t: &str| Oid::parse(t).map_err(|e| StoreError(format!("stored notice: {e}")));
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_application::{Notice, RemoteName};
    use dam_domain::{Object, Oid, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    #[test]
    fn an_object_round_trips_through_json() {
        let o = Object::Task(Task::new(oid(1), "milk"));
        assert_eq!(object_from_json(&object_to_json(&o).unwrap()).unwrap(), o);
    }

    #[test]
    fn ops_round_trip_and_bad_text_is_an_error() {
        for op in [Op::Create, Op::Update, Op::Delete] {
            assert_eq!(op_from_text(op_to_text(op)).unwrap(), op);
        }
        assert!(op_from_text("bogus").is_err());
    }

    #[test]
    fn every_notice_variant_round_trips() {
        let notices = vec![
            Notice::RemovedUpstream {
                remote: RemoteName("t".into()),
                oid: oid(1),
                subject: "s".into(),
            },
            Notice::EventCancelled {
                oid: oid(2),
                subject: "e".into(),
                attached: 3,
            },
            Notice::PushFailed {
                remote: RemoteName("t".into()),
                oid: oid(3),
                why: "w".into(),
            },
        ];
        for n in notices {
            assert_eq!(notice_from_json(&notice_to_json(&n).unwrap()).unwrap(), n);
        }
    }
}
