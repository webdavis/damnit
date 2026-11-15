#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Owed {
    Nothing,
    CommittedNotPushed,
    Unanswered,
    Refused,
    CreateAcceptedFollowUpUnanswered,
    DeleteUnanswered,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Here {
    Kept,
    RemovedNotCommitted,
    RemovedCommitted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Family {
    Alone,
    ParentOfBob,
    ChildOfAdaDeletedAfter,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Row {
    pub(super) owed: Owed,
    pub(super) here: Here,
    pub(super) family: Family,
    pub(super) edited_after: bool,
}

impl Row {
    fn can_occur(self) -> bool {
        let deleted_by_dam = self.owed == Owed::DeleteUnanswered;
        let created_by_dam = self.owed == Owed::CreateAcceptedFollowUpUnanswered;
        let a_parent_upstream_first = self.family == Family::ParentOfBob;
        let gone_after = self.family == Family::ChildOfAdaDeletedAfter;
        !(deleted_by_dam && self.here != Here::RemovedCommitted)
            && !((deleted_by_dam || created_by_dam) && a_parent_upstream_first)
            && !(self.edited_after && (self.here != Here::Kept || gone_after))
    }
}

pub(super) fn every_row() -> Vec<Row> {
    use Family::*;
    use Here::*;
    use Owed::*;
    let mut rows = Vec::new();
    for owed in [
        Nothing,
        CommittedNotPushed,
        Unanswered,
        Refused,
        CreateAcceptedFollowUpUnanswered,
        DeleteUnanswered,
    ] {
        for here in [Kept, RemovedNotCommitted, RemovedCommitted] {
            for family in [Alone, ParentOfBob, ChildOfAdaDeletedAfter] {
                for edited_after in [false, true] {
                    rows.push(Row {
                        owed,
                        here,
                        family,
                        edited_after,
                    });
                }
            }
        }
    }
    rows.retain(|r| r.can_occur());
    rows
}
