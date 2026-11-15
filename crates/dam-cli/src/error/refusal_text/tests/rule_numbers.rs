use dam_application::Refusal;

pub(super) const RULES: usize = 33;

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
        Refusal::MoveInsideItself(_) => 14,
        Refusal::NothingToCommit => 15,
        Refusal::NeedsAnAnswer => 16,
        Refusal::NeedsAnEditor => 17,
        Refusal::UnresolvedConflicts(_) => 18,
        Refusal::MissingCredential { .. } => 19,
        Refusal::StaleRemote { .. } => 20,
        Refusal::UnreadableRecurrence { .. } => 21,
        Refusal::NoConflict(_) => 22,
        Refusal::Token(_) => 23,
        Refusal::NotPermitted(_) => 24,
        Refusal::NotOpenToTokens(_) => 25,
        Refusal::SubjectNamesNoFolder(_) => 26,
        Refusal::FocusFull { .. } => 27,
        Refusal::TokenExists(_) => 28,
        Refusal::NotGrantable(_) => 29,
        Refusal::NoSuchToken(_) => 30,
        Refusal::NoSuchSetting(_) => 31,
        Refusal::Item(_) => 32,
    }
}
