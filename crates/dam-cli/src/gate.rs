use dam_application::{Access, Refusal};
use dam_domain::{Permission, Subject};

use crate::args::{Command, RemoteCommand, TokenCommand};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Need {
    OwnerOnly(&'static str),
    Whole(&'static [Permission]),
    NarrowedFrom(&'static [Permission]),
}

pub(crate) fn need(command: &Command) -> Need {
    use Permission::*;
    match command {
        Command::New(_) => Need::NarrowedFrom(&[TaskCreate, EventCreate]),
        Command::Done(_) => Need::NarrowedFrom(&[TaskComplete]),
        Command::Edit(a) if a.editor => Need::OwnerOnly("edit -e"),
        Command::Edit(_) => Need::NarrowedFrom(&[TaskChange, EventChange]),
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
        Command::Token(a) => match a.command {
            TokenCommand::Create(_) => Need::OwnerOnly("token create"),
            TokenCommand::List => Need::OwnerOnly("token list"),
            TokenCommand::Revoke { .. } => Need::OwnerOnly("token revoke"),
        },
    }
}

pub(crate) fn refuse_any_token_to_an_owner_only_verb(
    token_presented: bool,
    command: &Command,
) -> Result<(), Refusal> {
    match need(command) {
        Need::OwnerOnly(verb) if token_presented => Err(Refusal::NotOpenToTokens(verb)),
        _ => Ok(()),
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
