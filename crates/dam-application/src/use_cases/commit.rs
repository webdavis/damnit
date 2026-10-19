use dam_domain::{Change, CommitId, CommitRecord};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, Randomness};
use crate::service::Dam;

impl Dam<'_> {
    pub fn commit(&self, message: &str) -> Result<CommitRecord, UseCaseError> {
        let changes = self.store.staged()?;
        if changes.is_empty() {
            return Err(Refusal::NothingToCommit.into());
        }
        let record = record(self.clock, self.random, message.to_string(), changes);
        self.store.commit(&record)?;
        Ok(record)
    }
}

pub(crate) fn record(
    clock: &dyn Clock,
    random: &dyn Randomness,
    message: String,
    changes: Vec<Change>,
) -> CommitRecord {
    CommitRecord {
        id: CommitId::generate(&mut |b| random.fill(b)),
        message,
        at: clock.now(),
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::prelude::*;
    use crate::testing::{Scenario, oid};
    use dam_domain::{Object, Task};

    #[test]
    fn commit_records_the_stage_and_empties_it() {
        let s = Scenario::default().drawing_from(5);
        s.working(Object::Task(Task::new(oid(1), "a")));
        let dam = s.dam();
        dam.add_all().unwrap();
        let rec = dam.commit("first").unwrap();
        assert_eq!(rec.message, "first");
        assert_eq!(rec.changes.len(), 1);
        assert!(s.store.staged().unwrap().is_empty());
        assert_eq!(
            s.store.committed(&oid(1)).unwrap().unwrap().base().subject,
            "a"
        );
        assert_eq!(s.store.log().unwrap()[0].id, rec.id);
    }

    #[test]
    fn an_empty_stage_cannot_be_committed() {
        let s = Scenario::default().drawing_from(5);
        let err = s.dam().commit("x").unwrap_err();
        assert_eq!(err, UseCaseError::Refused(Refusal::NothingToCommit));
    }
}
