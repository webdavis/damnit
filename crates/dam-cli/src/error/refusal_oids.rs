use dam_application::{ItemRefusal, Refusal};
use dam_domain::Oid;

pub(super) fn refusal_oids(refusal: &Refusal) -> Vec<String> {
    let named = |oid: &Oid| oid.to_string();
    match refusal {
        Refusal::Blocked { oid, blockers } => std::iter::once(named(oid))
            .chain(blockers.dependencies.iter().map(named))
            .chain(blockers.children.iter().map(named))
            .collect(),
        Refusal::Cycle { oid, path } => std::iter::once(named(oid))
            .chain(path.iter().map(named))
            .collect(),
        Refusal::AmbiguousPrefix { matches, .. } => matches.clone(),
        Refusal::FocusFull { focused, .. } => focused.iter().map(named).collect(),
        Refusal::NotATask(oid)
        | Refusal::NotAnEvent(oid)
        | Refusal::MoveInsideItself(oid)
        | Refusal::SubjectNamesNoFolder(oid)
        | Refusal::NotCompleted(oid)
        | Refusal::NotCommitted(oid)
        | Refusal::NoConflict(oid)
        | Refusal::DirtyOnPull { oid } => vec![named(oid)],
        Refusal::Labels(_)
        | Refusal::UnknownCategory(_)
        | Refusal::NotACategoryValue { .. }
        | Refusal::NoSuchObject(_)
        | Refusal::NoWorkingObject(_)
        | Refusal::NoSuchRemote(_)
        | Refusal::RemoteExists(_)
        | Refusal::UnresolvedConflicts(_)
        | Refusal::MissingCredential { .. }
        | Refusal::StaleRemote { .. }
        | Refusal::UnreadableRecurrence { .. }
        | Refusal::NothingToCommit
        | Refusal::NeedsAnAnswer
        | Refusal::NeedsAnEditor
        | Refusal::Token(_)
        | Refusal::NotPermitted(_)
        | Refusal::NotOpenToTokens(_)
        | Refusal::TokenExists(_)
        | Refusal::NotGrantable(_)
        | Refusal::NoSuchToken(_) => Vec::new(),
        Refusal::Item(refusal) => item_oids(refusal),
    }
}

fn item_oids(refusal: &ItemRefusal) -> Vec<String> {
    match refusal {
        ItemRefusal::Declined(hash)
        | ItemRefusal::NotAnObject(hash)
        | ItemRefusal::NotYours(hash)
        | ItemRefusal::NotWaiting { hash, .. } => vec![hash.to_string()],
        ItemRefusal::TargetGone { hash, target }
        | ItemRefusal::TargetNotCommitted { hash, target }
        | ItemRefusal::TargetStaged { hash, target }
        | ItemRefusal::EditedSinceAdded { hash, target }
        | ItemRefusal::SubjectNamesNoFolder { hash, target }
        | ItemRefusal::Clash { hash, target, .. } => vec![hash.to_string(), target.to_string()],
        ItemRefusal::NothingChanged => Vec::new(),
    }
}
