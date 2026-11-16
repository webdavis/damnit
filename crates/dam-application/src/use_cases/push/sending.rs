use crate::errors::UseCaseError;
use crate::ports::{Progress, RemoteName, Step};
use crate::service::Dam;
use crate::use_cases::push::waves::Sent;

impl Dam<'_> {
    pub(super) fn sending(
        &self,
        remote: &RemoteName,
        changes: usize,
        send: impl FnOnce(&dyn Progress) -> Result<Sent, UseCaseError>,
    ) -> Result<Sent, UseCaseError> {
        let progress = self.shown_progress();
        if changes == 0 {
            return send(progress);
        }
        progress.start(Step {
            doing: &format!("sending to {}", remote.0),
            did: &format!("sent to {}", remote.0),
            unit: "changes",
            total: Some(changes),
        });
        let sent = send(progress);
        match &sent {
            Ok(Sent {
                stopped_by: None, ..
            }) => progress.finish(),
            _ => progress.fail(),
        }
        sent
    }
}
