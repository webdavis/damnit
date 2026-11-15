use dam_application::{
    DroppedBecause, MissingTask, Notice, RejectedObject, RemoteName, StoreError,
};
use dam_domain::{Kind, Oid, Path};
use serde::{Deserialize, Serialize};

mod task_pulled;

use task_pulled::TaskPulledFailedRow;

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
    PullSkipped {
        remote: String,
        remote_id: String,
        why: String,
    },
    KindChanged {
        oid: String,
        ours: String,
        theirs: String,
    },
    MissingUpstream {
        remote: String,
        tasks: Vec<MissingTaskRow>,
    },
    UnansweredCreateDropped {
        remote: String,
        oid: String,
        subject: String,
        path: String,
        because: String,
    },
    TaskPulledFailed(TaskPulledFailedRow),
}

#[derive(Serialize, Deserialize)]
struct MissingTaskRow {
    oid: String,
    subject: String,
}

pub(in crate::sqlite) fn notice_to_json(n: &Notice) -> Result<String, StoreError> {
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
        Notice::PullSkipped { remote, object } => NoticeRow::PullSkipped {
            remote: remote.0.clone(),
            remote_id: object.remote_id.clone(),
            why: object.why.clone(),
        },
        Notice::KindChanged { oid, ours, theirs } => NoticeRow::KindChanged {
            oid: oid.to_string(),
            ours: ours.as_str().to_string(),
            theirs: theirs.as_str().to_string(),
        },
        Notice::MissingUpstream { remote, tasks } => NoticeRow::MissingUpstream {
            remote: remote.0.clone(),
            tasks: tasks
                .iter()
                .map(|t| MissingTaskRow {
                    oid: t.oid.to_string(),
                    subject: t.subject.clone(),
                })
                .collect(),
        },
        Notice::UnansweredCreateDropped {
            remote,
            oid,
            subject,
            path,
            because,
        } => NoticeRow::UnansweredCreateDropped {
            remote: remote.0.clone(),
            oid: oid.to_string(),
            subject: subject.clone(),
            path: path.to_string(),
            because: dropped_because_name(*because).into(),
        },
        Notice::TaskPulledFailed {
            remote,
            failed,
            not_reached,
            error,
        } => NoticeRow::TaskPulledFailed(TaskPulledFailedRow::new(
            remote,
            *failed,
            *not_reached,
            error.as_ref(),
        )),
    };
    serde_json::to_string(&row).map_err(|e| StoreError::Failed(format!("stored notice: {e}")))
}

pub(in crate::sqlite) fn notice_from_json(s: &str) -> Result<Notice, StoreError> {
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
        NoticeRow::PullSkipped {
            remote,
            remote_id,
            why,
        } => Notice::PullSkipped {
            remote: RemoteName(remote),
            object: RejectedObject { remote_id, why },
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
        NoticeRow::MissingUpstream { remote, tasks } => Notice::MissingUpstream {
            remote: RemoteName(remote),
            tasks: tasks
                .into_iter()
                .map(|t| {
                    Ok(MissingTask {
                        oid: oid(&t.oid)?,
                        subject: t.subject,
                    })
                })
                .collect::<Result<_, StoreError>>()?,
        },
        NoticeRow::UnansweredCreateDropped {
            remote,
            oid: o,
            subject,
            path,
            because,
        } => Notice::UnansweredCreateDropped {
            remote: RemoteName(remote),
            oid: oid(&o)?,
            subject,
            path: Path::parse(&path)
                .map_err(|e| StoreError::Failed(format!("stored notice: {e}")))?,
            because: dropped_because(&because)?,
        },
        NoticeRow::TaskPulledFailed(row) => row.notice(),
    })
}

fn dropped_because_name(because: DroppedBecause) -> &'static str {
    match because {
        DroppedBecause::Removed => "removed",
        DroppedBecause::NoLongerSent => "no_longer_sent",
    }
}

fn dropped_because(name: &str) -> Result<DroppedBecause, StoreError> {
    [DroppedBecause::Removed, DroppedBecause::NoLongerSent]
        .into_iter()
        .find(|b| dropped_because_name(*b) == name)
        .ok_or_else(|| StoreError::Failed(format!("stored notice: dropped because {name:?}")))
}
