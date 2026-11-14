use dam_application::Notice;
use dam_domain::Oid;
use serde::Serialize;

use crate::view::display;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum NoticeRow<'a> {
    RemovedUpstream {
        remote: &'a str,
        #[serde(serialize_with = "display")]
        oid: &'a Oid,
        subject: &'a str,
    },
    EventCancelled {
        #[serde(serialize_with = "display")]
        oid: &'a Oid,
        subject: &'a str,
        attached: usize,
    },
    PushFailed {
        remote: &'a str,
        #[serde(serialize_with = "display")]
        oid: &'a Oid,
        why: &'a str,
    },
    PullFailed {
        remote: &'a str,
        why: &'a str,
    },
    PullSkipped {
        remote: &'a str,
        remote_id: &'a str,
        why: &'a str,
    },
    KindChanged {
        #[serde(serialize_with = "display")]
        oid: &'a Oid,
        ours: &'static str,
        theirs: &'static str,
    },
    MissingUpstream {
        remote: &'a str,
        tasks: Vec<MissingTaskRow<'a>>,
    },
}

#[derive(Serialize)]
pub(super) struct MissingTaskRow<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    subject: &'a str,
}

impl<'a> NoticeRow<'a> {
    pub(super) fn new(n: &'a Notice) -> NoticeRow<'a> {
        match n {
            Notice::RemovedUpstream {
                remote,
                oid,
                subject,
            } => NoticeRow::RemovedUpstream {
                remote: &remote.0,
                oid,
                subject,
            },
            Notice::EventCancelled {
                oid,
                subject,
                attached,
            } => NoticeRow::EventCancelled {
                oid,
                subject,
                attached: *attached,
            },
            Notice::PushFailed { remote, oid, why } => NoticeRow::PushFailed {
                remote: &remote.0,
                oid,
                why,
            },
            Notice::PullFailed { remote, why } => NoticeRow::PullFailed {
                remote: &remote.0,
                why,
            },
            Notice::PullSkipped { remote, object } => NoticeRow::PullSkipped {
                remote: &remote.0,
                remote_id: &object.remote_id,
                why: &object.why,
            },
            Notice::KindChanged { oid, ours, theirs } => NoticeRow::KindChanged {
                oid,
                ours: ours.as_str(),
                theirs: theirs.as_str(),
            },
            Notice::MissingUpstream { remote, tasks } => NoticeRow::MissingUpstream {
                remote: &remote.0,
                tasks: tasks
                    .iter()
                    .map(|t| MissingTaskRow {
                        oid: &t.oid,
                        subject: &t.subject,
                    })
                    .collect(),
            },
        }
    }
}

mod text;
