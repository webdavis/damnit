//! One notice as a line the operator reads and as the object a client reads.
//! A notice is something upstream did that dam reports and never acts on.

use dam_application::Notice;

pub(super) fn notice_line(n: &Notice) -> String {
    match n {
        Notice::RemovedUpstream {
            remote,
            oid,
            subject,
        } => format!(
            "{} was removed on {}: {subject:?} is kept here; dam rm {} to drop it",
            oid.short(),
            remote.0,
            oid.short()
        ),
        Notice::EventCancelled {
            oid,
            subject,
            attached,
        } => format!(
            "event {} {subject:?} was cancelled; {attached} attached task(s) kept",
            oid.short()
        ),
        Notice::PushFailed { remote, oid, why } => {
            format!("push of {} to {} failed: {why}", oid.short(), remote.0)
        }
        Notice::PullFailed { remote, why } => {
            format!("pull from {} failed: {why}", remote.0)
        }
        Notice::KindChanged { oid, ours, theirs } => format!(
            "{} is {} {ours} here and {} {theirs} upstream",
            oid.short(),
            article(ours.as_str()),
            article(theirs.as_str())
        ),
    }
}

/// The article a kind word takes, so a reworded or added kind still reads.
fn article(word: &str) -> &'static str {
    match word.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

pub(super) fn notice_json(n: &Notice) -> serde_json::Value {
    match n {
        Notice::RemovedUpstream {
            remote,
            oid,
            subject,
        } => {
            serde_json::json!({ "kind": "removed_upstream", "remote": remote.0, "oid": oid.to_string(), "subject": subject })
        }
        Notice::EventCancelled {
            oid,
            subject,
            attached,
        } => {
            serde_json::json!({ "kind": "event_cancelled", "oid": oid.to_string(), "subject": subject, "attached": attached })
        }
        Notice::PushFailed { remote, oid, why } => {
            serde_json::json!({ "kind": "push_failed", "remote": remote.0, "oid": oid.to_string(), "why": why })
        }
        Notice::PullFailed { remote, why } => {
            serde_json::json!({ "kind": "pull_failed", "remote": remote.0, "why": why })
        }
        Notice::KindChanged { oid, ours, theirs } => {
            serde_json::json!({ "kind": "kind_changed", "oid": oid.to_string(), "ours": ours.as_str(), "theirs": theirs.as_str() })
        }
    }
}
