use dam_application::Refusal;

pub(super) const RULES: usize = 32;

pub(super) fn rule_number(refusal: &Refusal) -> usize {
    match refusal {
        Refusal::Blocked { .. } => 0,
        Refusal::Cycle { .. } => 1,
        Refusal::Labels(_) => 2,
        Refusal::UnknownCategory(_) => 3,
        Refusal::NotACategoryValue { .. } => 4,
        Refusal::NoSuchObject(_) => 5,
        Refusal::NoWorkingObject(_) => 6,
        Refusal::AmbiguousPrefix { .. } => 7,
        Refusal::NoSuchRemote(_) => 8,
        Refusal::RemoteExists(_) => 9,
        Refusal::NotATask(_) => 10,
        Refusal::NotAnEvent(_) => 11,
        Refusal::NotCompleted(_) => 12,
        Refusal::NotCommitted(_) => 13,
        Refusal::DirtyOnPull { .. } => 14,
        Refusal::MoveInsideItself(_) => 15,
        Refusal::NothingToCommit => 16,
        Refusal::NeedsAnAnswer => 17,
        Refusal::NeedsAnEditor => 18,
        Refusal::UnresolvedConflicts(_) => 19,
        Refusal::MissingCredential { .. } => 20,
        Refusal::StaleRemote { .. } => 21,
        Refusal::UnreadableRecurrence { .. } => 22,
        Refusal::NoConflict(_) => 23,
        Refusal::Token(_) => 24,
        Refusal::NotPermitted(_) => 25,
        Refusal::NotOpenToTokens(_) => 26,
        Refusal::SubjectNamesNoFolder(_) => 27,
        Refusal::FocusFull { .. } => 28,
        Refusal::TokenExists(_) => 29,
        Refusal::NotGrantable(_) => 30,
        Refusal::NoSuchToken(_) => 31,
    }
}
