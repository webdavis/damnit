use dam_application::{Access, Refusal};
use dam_domain::{Permission, Subject};

use crate::args::{Command, RemoteCommand};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Need {
    OwnerOnly(&'static str),
    Whole(&'static [Permission]),
    NarrowedFrom(&'static [Permission]),
}

pub(crate) fn need(command: &Command) -> Need {
    use Permission::*;
    match command {
        Command::New(_) => Need::OwnerOnly("new"),
        Command::Done(_) => Need::OwnerOnly("done"),
        Command::Edit(_) => Need::OwnerOnly("edit"),
        Command::Mv(_) => Need::OwnerOnly("mv"),
        Command::Rm(_) => Need::OwnerOnly("rm"),
        Command::Add(_) => Need::OwnerOnly("add"),
        Command::Reset(_) => Need::OwnerOnly("reset"),
        Command::Restore(_) => Need::OwnerOnly("restore"),
        Command::Commit(_) => Need::OwnerOnly("commit"),
        Command::Log => Need::Whole(&[HistoryRead]),
        Command::Status(_) | Command::Diff(_) => Need::Whole(&[WorkingRead, StageRead]),
        Command::Category(_) | Command::Filter(_) => Need::Whole(&[ConfigRead]),
        Command::Ls(_) => Need::NarrowedFrom(&[TaskRead, EventRead]),
        Command::Agenda(_) => Need::NarrowedFrom(&[EventRead]),
        Command::Reminders(_) => Need::NarrowedFrom(&[TaskRead, EventRead]),
        Command::Show(_) => Need::NarrowedFrom(&[TaskRead, EventRead, HistoryRead]),
        Command::Remote(a) => match a.command {
            RemoteCommand::Add { .. } => Need::OwnerOnly("remote add"),
            RemoteCommand::List => Need::NarrowedFrom(&[RemoteRead]),
        },
        Command::Push(_) => Need::NarrowedFrom(&[RemotePush]),
        Command::Pull(_) => Need::NarrowedFrom(&[RemotePull]),
        Command::Resolve(_) => Need::OwnerOnly("resolve"),
        Command::Day(_) => Need::OwnerOnly("day"),
    }
}

pub(crate) fn admit(access: &dyn Access, command: &Command) -> Result<(), Refusal> {
    match need(command) {
        Need::OwnerOnly(verb) if access.token().is_some() => Err(Refusal::NotOpenToTokens(verb)),
        Need::OwnerOnly(_) => Ok(()),
        Need::Whole(permissions) => match permissions
            .iter()
            .find(|p| !access.permits(**p, &[Subject::Whole]))
        {
            Some(missing) => Err(Refusal::NotPermitted(*missing)),
            None => Ok(()),
        },
        Need::NarrowedFrom(permissions) if permissions.iter().any(|p| access.holds(*p)) => Ok(()),
        Need::NarrowedFrom(permissions) => Err(Refusal::NotPermitted(permissions[0])),
    }
}

pub(crate) fn may(access: &dyn Access, permission: Permission) -> bool {
    access.permits(permission, &[Subject::Whole])
}

#[cfg(test)]
mod tests;
