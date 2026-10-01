use dam_application::Refusal;
use dam_domain::{Blocker, Oid};

pub(super) fn refusal_oids(refusal: &Refusal) -> Vec<String> {
    let named = |oid: &Oid| oid.to_string();
    match refusal {
        Refusal::Blocked { oid, blockers } => std::iter::once(named(oid))
            .chain(blockers.iter().map(|b| named(blocked_by(b))))
            .collect(),
        Refusal::Cycle { oid, path } => std::iter::once(named(oid))
            .chain(path.iter().map(named))
            .collect(),
        Refusal::NotATask(oid)
        | Refusal::NotAnEvent(oid)
        | Refusal::MoveInsideItself(oid)
        | Refusal::NotCompleted(oid)
        | Refusal::NotCommitted(oid)
        | Refusal::DirtyOnPull { oid } => vec![named(oid)],
        Refusal::Labels(_)
        | Refusal::UnknownCategory(_)
        | Refusal::NoSuchObject(_)
        | Refusal::NoWorkingObject(_)
        | Refusal::NoSuchRemote(_)
        | Refusal::UnresolvedConflicts(_)
        | Refusal::MissingCredential { .. }
        | Refusal::StaleRemote { .. }
        | Refusal::NothingToCommit
        | Refusal::NeedsAnAnswer
        | Refusal::NeedsAnEditor => Vec::new(),
    }
}

fn blocked_by(blocker: &Blocker) -> &Oid {
    match blocker {
        Blocker::OpenDependency(oid) | Blocker::OpenChild(oid) => oid,
    }
}
