use dam_application::Refusal;
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
        Refusal::NotATask(oid)
        | Refusal::NotAnEvent(oid)
        | Refusal::MoveInsideItself(oid)
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
        | Refusal::NeedsAnEditor => Vec::new(),
    }
}
