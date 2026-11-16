use dam_application::{Access, Refusal};
use dam_domain::{Permission, Subject};

use crate::args::hook::HookCommand;

use crate::args::{
    CategoryCommand, Command, ConfigCommand, FilterCommand, RemoteCommand, SyncCommand,
    TokenCommand,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Need {
    Open,
    OwnerOnly(&'static str),
    Whole(&'static [Permission]),
    NarrowedFrom(&'static [Permission]),
}

pub(crate) fn need(command: &Command) -> Need {
    use Permission::*;
    match command {
        Command::Init(_) => Need::OwnerOnly("init"),
        Command::New(_) => Need::NarrowedFrom(&[TaskCreate, EventCreate]),
        Command::Done(_) => Need::NarrowedFrom(&[TaskComplete]),
        Command::Edit(a) if a.editor => Need::OwnerOnly("edit -e"),
        Command::Edit(_) => Need::NarrowedFrom(&[TaskChange, EventChange]),
        Command::Mv(_) => Need::OwnerOnly("mv"),
        Command::Rm(_) => Need::Open,
        Command::Add(_) => Need::OwnerOnly("add"),
        Command::Reset(_) => Need::OwnerOnly("reset"),
        Command::Restore(_) => Need::OwnerOnly("restore"),
        Command::Revert(_) => Need::OwnerOnly("revert"),
        Command::Commit(_) => Need::OwnerOnly("commit"),
        Command::Log => Need::Whole(&[HistoryRead]),
        Command::Status(_) => Need::Open,
        Command::Diff(_) => Need::Whole(&[WorkingRead, StageRead]),
        Command::Category(a) => match a.command {
            CategoryCommand::List => Need::Whole(&[ConfigRead]),
            CategoryCommand::Create { .. } => Need::OwnerOnly("category create"),
            CategoryCommand::Remove { .. } => Need::OwnerOnly("category remove"),
            CategoryCommand::Migrate { .. } => Need::OwnerOnly("category migrate"),
        },
        Command::Filter(a) => match a.command {
            FilterCommand::List => Need::Whole(&[ConfigRead]),
            FilterCommand::Create { .. } => Need::OwnerOnly("filter create"),
            FilterCommand::Remove { .. } => Need::OwnerOnly("filter remove"),
        },
        Command::List(_) | Command::Label(_) => Need::NarrowedFrom(&[TaskRead, EventRead]),
        Command::Agenda(_) => Need::NarrowedFrom(&[EventRead]),
        Command::Reminders(_) => Need::NarrowedFrom(&[TaskRead, EventRead]),
        Command::Show(_) => Need::NarrowedFrom(&[TaskRead, EventRead, HistoryRead]),
        Command::Hook(a) => match a.command {
            None => Need::Open,
            Some(HookCommand::List) => Need::OwnerOnly("hook list"),
            Some(HookCommand::Run(_)) => Need::OwnerOnly("hook run"),
        },
        Command::Remote(a) => match a.command {
            RemoteCommand::Add { .. } => Need::OwnerOnly("remote add"),
            RemoteCommand::List => Need::NarrowedFrom(&[RemoteRead]),
        },
        Command::Push(_) => Need::NarrowedFrom(&[RemotePush]),
        Command::Pull(_) => Need::NarrowedFrom(&[RemotePull]),
        Command::Resolve(_) => Need::OwnerOnly("resolve"),
        Command::Sync(a) => match a.command {
            None => Need::Open,
            Some(SyncCommand::Run { .. }) => Need::NarrowedFrom(&[RemotePull, RemotePush]),
            Some(SyncCommand::Start { .. }) => Need::OwnerOnly("sync start"),
            Some(SyncCommand::Stop { .. }) => Need::OwnerOnly("sync stop"),
            Some(SyncCommand::Enable { .. }) => Need::OwnerOnly("sync enable"),
            Some(SyncCommand::Disable { .. }) => Need::OwnerOnly("sync disable"),
        },
        Command::Day(_) => Need::OwnerOnly("day"),
        Command::Token(a) => match a.command {
            TokenCommand::Create(_) => Need::OwnerOnly("token create"),
            TokenCommand::List => Need::OwnerOnly("token list"),
            TokenCommand::Revoke { .. } => Need::OwnerOnly("token revoke"),
        },
        Command::Config(a) => match a.command {
            None => Need::Open,
            Some(ConfigCommand::List) => Need::OwnerOnly("config list"),
            Some(ConfigCommand::Get { .. }) => Need::OwnerOnly("config get"),
            Some(ConfigCommand::Set { .. }) => Need::OwnerOnly("config set"),
            Some(ConfigCommand::Unset { .. }) => Need::OwnerOnly("config unset"),
            Some(ConfigCommand::PrintDefault) => Need::OwnerOnly("config print-default"),
            Some(ConfigCommand::Validate) => Need::OwnerOnly("config validate"),
        },
        Command::Doctor(_) => Need::OwnerOnly("doctor"),
        Command::Test(_) => Need::OwnerOnly("test"),
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
        Need::Open => Ok(()),
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
mod background_tests;
#[cfg(test)]
mod tests;
