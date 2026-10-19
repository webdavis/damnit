use crate::errors::UseCaseError;
use crate::ports::{Notice, NoticeRepository, RemoteName};
use crate::remote::{IncomingObject, RejectedObject};

pub(super) fn record_skipped(
    notices: &dyn NoticeRepository,
    remote: &RemoteName,
    read_cleanly: &[IncomingObject],
    rejected: &[RejectedObject],
) -> Result<(), UseCaseError> {
    for stale in notices.notices()?.iter().filter(|n| {
        matches!(n, Notice::PullSkipped { remote: r, object }
            if r == remote && read_cleanly.iter().any(|o| o.remote_id == object.remote_id))
    }) {
        notices.remove_notice(stale)?;
    }
    let held = notices.notices()?;
    for object in rejected {
        let notice = Notice::PullSkipped {
            remote: remote.clone(),
            object: object.clone(),
        };
        if !held.contains(&notice) {
            notices.add_notice(&notice)?;
        }
    }
    Ok(())
}
