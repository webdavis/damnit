use crate::ports::{NoProgress, Progress, Step};
use crate::service::Dam;

pub(crate) struct Words<'a> {
    pub(crate) doing: &'a str,
    pub(crate) did: &'a str,
}

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
        words: Words<'_>,
        wait: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let progress = self.shown_progress();
        progress.start(Step {
            doing: words.doing,
            did: words.did,
            unit: "",
            total: None,
        });
        let answer = wait();
        match &answer {
            Ok(_) => progress.finish(),
            Err(_) => progress.fail(),
        }
        answer
    }
}
