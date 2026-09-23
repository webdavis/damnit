use std::fmt;

use dam_domain::Blocker;

use super::Refusal;

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Blocked { oid, blockers } => {
                writeln!(f, "{} cannot be completed:", oid.short())?;
                for b in blockers {
                    match b {
                        Blocker::OpenDependency(d) => {
                            writeln!(f, "  depends on {} which is open", d.short())?
                        }
                        Blocker::OpenChild(c) => writeln!(f, "  child {} is open", c.short())?,
                    }
                }
                f.write_str("use --force to complete it anyway, or --force --interactive to decide what happens to them")
            }
            Refusal::Cycle { oid, path } => {
                let chain: Vec<&str> = path.iter().map(|o| o.short()).collect();
                write!(
                    f,
                    "{} cannot depend on that: it would form a cycle through {}",
                    oid.short(),
                    chain.join(" -> ")
                )
            }
            Refusal::Labels(v) => write!(f, "{v}"),
            Refusal::UnknownCategory(n) => write!(f, "{n:?} is not a declared category"),
            Refusal::NoSuchObject(s) => write!(f, "no object matches {s:?}"),
            Refusal::NoWorkingObject(s) => write!(
                f,
                "no object in the working layer matches {s:?}; one removed from it is named by its full oid"
            ),
            Refusal::NoSuchRemote(s) => write!(f, "no remote named {s:?}"),
            Refusal::NotATask(o) => {
                write!(f, "{} is an event; events are not completed", o.short())
            }
            Refusal::NotAnEvent(o) => write!(
                f,
                "{} is a task; start, end and location are event fields",
                o.short()
            ),
            Refusal::NotCompleted(o) => write!(
                f,
                "{} is not completed, so there is nothing to reopen",
                o.short()
            ),
            Refusal::NotCommitted(o) => write!(
                f,
                "{} has no commit behind it; dam rm removes it instead",
                o.short()
            ),
            Refusal::DirtyOnPull { oid } => write!(
                f,
                "{} changed upstream and has uncommitted local changes; commit or reset it, then pull again",
                oid.short()
            ),
            Refusal::UnresolvedConflicts(n) => {
                write!(f, "{n} conflicts are unresolved; run dam resolve")
            }
            Refusal::MoveInsideItself(o) => {
                write!(f, "{} cannot move inside itself", o.short())
            }
            Refusal::NothingToCommit => f.write_str("nothing to commit"),
            Refusal::NeedsAnAnswer => f.write_str(
                "a question needs an answer; drop --json/--toon to answer interactively",
            ),
            Refusal::NeedsAnEditor => {
                f.write_str("-e opens an editor; drop --json/--toon to use it")
            }
            Refusal::MissingCredential { remote, name } => {
                write!(
                    f,
                    "remote {remote:?} needs {name}; set {name}, {name}_command or {name}_env in its config"
                )
            }
        }
    }
}
