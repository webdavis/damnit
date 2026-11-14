use super::commands::Command;
use super::shape::Shape;
use crate::api::{ApiError, NotAccepted, SyncWrite};

pub enum Failure {
    OnlyThisMutation(String),
    Unanswered(String),
    Unreachable(String),
    StopsThePush(ApiError),
}

const FIRST_SERVER_ERROR: u16 = 500;

impl From<ApiError> for Failure {
    fn from(e: ApiError) -> Failure {
        match e {
            rate @ ApiError::RateLimited { .. } => Failure::StopsThePush(rate),
            ApiError::Http { status, .. } if status < FIRST_SERVER_ERROR => {
                Failure::OnlyThisMutation(e.to_string())
            }
            ApiError::Http { status, body, .. } => Failure::Unanswered(format!(
                "Todoist answered with a server error, {status}: {body}"
            )),
            ApiError::Transport { why, .. } => Failure::Unanswered(format!(
                "the connection to Todoist was lost before it answered: {why}"
            )),
            ApiError::Decode { why, .. } => {
                Failure::Unanswered(format!("Todoist's answer could not be read: {why}"))
            }
            ApiError::TooLarge { limit, .. } => Failure::Unanswered(format!(
                "Todoist's answer was past {limit} bytes and could not be read"
            )),
            ApiError::TimedOut { .. } => Failure::Unanswered(e.to_string()),
            ApiError::Unreachable { .. } => Failure::Unreachable(e.to_string()),
        }
    }
}

impl From<String> for Failure {
    fn from(why: String) -> Failure {
        Failure::OnlyThisMutation(why)
    }
}

pub fn every_command_accepted(commands: &[Command], written: &SyncWrite) -> Result<(), Failure> {
    let mut accepted = Vec::new();
    let mut refused = Vec::new();
    for sent in commands {
        match written.accepted(&sent.uuid) {
            Ok(()) => accepted.push(sent.kind),
            Err(NotAccepted::Refused(why)) => refused.push((sent.kind, why)),
            Err(NotAccepted::NoVerdict) => {
                let kind = sent.kind;
                return Err(Failure::Unanswered(format!(
                    "Todoist gave no verdict for {kind}"
                )));
            }
        }
    }
    match refused.into_iter().next() {
        None => Ok(()),
        Some((_, why)) if accepted.is_empty() => Err(Failure::OnlyThisMutation(why)),
        Some((kind, why)) => Err(Failure::Unanswered(format!(
            "Todoist accepted {} but refused {kind}: {why}",
            accepted.join(", ")
        ))),
    }
}

pub fn no_id_named_for(shape: &Shape) -> Failure {
    let noun = match shape {
        Shape::Project { .. } => "project",
        Shape::Section { .. } => "section",
        Shape::Item(_) => "task",
    };
    Failure::Unanswered(format!(
        "Todoist accepted the new {noun} but named no id for it"
    ))
}
