use super::NoticeRow;
use crate::layout::counted;

impl NoticeRow<'_> {
    pub(in crate::commands::status) fn lines(&self) -> Vec<String> {
        let mut lines = vec![self.line()];
        if let NoticeRow::MissingUpstream { tasks, .. } = self {
            lines.extend(
                tasks
                    .iter()
                    .map(|t| format!("  {} {:?}", t.oid.short(), t.subject)),
            );
        }
        lines
    }

    fn line(&self) -> String {
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
            NoticeRow::PullSkipped {
                remote,
                remote_id: "",
                why,
            } => format!("pull from {remote} skipped an object: {why}"),
            NoticeRow::PullSkipped {
                remote,
                remote_id,
                why,
            } => format!("pull from {remote} skipped {remote_id}: {why}"),
            NoticeRow::MissingUpstream { remote, tasks } => format!(
                "{remote} no longer lists {}; dam done <oid> if finished there, dam rm <oid> if deleted",
                counted(tasks.len(), "open task", "open tasks")
            ),
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
