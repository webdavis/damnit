use dam_application::Refusal;

pub(super) const RULES: usize = 34;

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
        Refusal::MissingCredential { .. } => 18,
        Refusal::StaleRemote { .. } => 19,
        Refusal::UnreadableRecurrence { .. } => 20,
        Refusal::NoConflict(_) => 21,
        Refusal::Token(_) => 22,
        Refusal::NotPermitted(_) => 23,
        Refusal::NotOpenToTokens(_) => 24,
        Refusal::SubjectNamesNoFolder(_) => 25,
        Refusal::FocusFull { .. } => 26,
        Refusal::TokenExists(_) => 27,
        Refusal::NotGrantable(_) => 28,
        Refusal::NoSuchToken(_) => 29,
        Refusal::NoSuchSetting(_) => 30,
        Refusal::Item(_) => 31,
        Refusal::PulledCommit(_) => 32,
        Refusal::NotInCommit { .. } => 33,
    }
}
