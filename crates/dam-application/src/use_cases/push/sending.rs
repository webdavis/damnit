use crate::errors::UseCaseError;
use crate::ports::{RemoteHelper, RemoteName, Step};
use crate::remote::RemoteMutation;
use crate::service::Dam;
use crate::use_cases::push::removals::Removals;
use crate::use_cases::push::waves::{Sent, send_in_waves};

impl Dam<'_> {
    pub(super) fn sending(
        &self,
        helper: &mut dyn RemoteHelper,
        remote: &RemoteName,
        removals: Removals,
        mutations: Vec<RemoteMutation>,
        record: &dyn Fn(&[RemoteMutation]) -> Result<(), UseCaseError>,
    ) -> Result<Sent, UseCaseError> {
        let progress = self.shown_progress();
        if mutations.is_empty() {
            return send_in_waves(helper, removals, mutations, record, progress);
        }
        progress.start(Step {
            doing: &format!("sending to {}", remote.0),
            did: &format!("sent to {}", remote.0),
            unit: "changes",
            total: Some(mutations.len()),
        });
        let sent = send_in_waves(helper, removals, mutations, record, progress);
        match &sent {
            Ok(Sent {
                stopped_by: None, ..
            }) => progress.finish(),
            _ => progress.fail(),
        }
        sent
    }
}
