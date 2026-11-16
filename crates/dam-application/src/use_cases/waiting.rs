use crate::ports::{NoProgress, Progress, Step, waited_on};
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn shown_progress(&self) -> &dyn Progress {
        if self.access.token().is_none() {
            self.progress
        } else {
            &NoProgress
        }
    }

    pub(crate) fn waiting_on<T, E>(
        &self,
        step: Step<'_>,
        wait: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        waited_on(self.shown_progress(), step, wait)
    }
}
