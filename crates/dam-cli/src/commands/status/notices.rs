use dam_application::Notice;
use dam_domain::Oid;
use serde::Serialize;

use crate::layout::counted;
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
    KindChanged {
        #[serde(serialize_with = "display")]
        oid: &'a Oid,
        ours: &'static str,
        theirs: &'static str,
    },
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
            Notice::KindChanged { oid, ours, theirs } => NoticeRow::KindChanged {
                oid,
                ours: ours.as_str(),
                theirs: theirs.as_str(),
            },
        }
    }

    pub(super) fn line(&self) -> String {
        match self {
            NoticeRow::RemovedUpstream {
                remote,
                oid,
                subject,
            } => format!(
                "{} was removed on {remote}: {subject:?} is kept here; dam rm {} to drop it",
                oid.short(),
                oid.short()
            ),
            NoticeRow::EventCancelled {
                oid,
                subject,
                attached,
            } => format!(
                "event {} {subject:?} was cancelled; {} kept",
                oid.short(),
                counted(*attached, "attached task", "attached tasks")
            ),
            NoticeRow::PushFailed { remote, oid, why } => {
                format!("push of {} to {remote} failed: {why}", oid.short())
            }
            NoticeRow::PullFailed { remote, why } => {
                format!("pull from {remote} failed: {why}")
            }
            NoticeRow::KindChanged { oid, ours, theirs } => format!(
                "{} is {} {ours} here and {} {theirs} upstream",
                oid.short(),
                article_a_reworded_or_added_kind_still_reads_with(ours),
                article_a_reworded_or_added_kind_still_reads_with(theirs)
            ),
        }
    }
}

fn article_a_reworded_or_added_kind_still_reads_with(word: &str) -> &'static str {
    match word.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}
