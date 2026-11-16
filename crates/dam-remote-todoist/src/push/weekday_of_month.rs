use std::borrow::Cow;

use dam_protocol::{Mutation, WireObject};

use super::Failure;
use crate::rule_words::{Unheld, to_todoist};

pub fn in_todoist_words<'o>(
    m: &Mutation,
    object: &'o WireObject,
) -> Result<Cow<'o, WireObject>, Failure> {
    let Some(rule) = &object.recurrence else {
        return Ok(Cow::Borrowed(object));
    };
    let sends_the_rule =
        m.op == "create" || m.fields.iter().any(|f| f == "due" || f == "recurrence");
    match to_todoist(rule) {
        Ok(words) => Ok(Cow::Owned(WireObject {
            recurrence: Some(words),
            ..object.clone()
        })),
        Err(_) if !sends_the_rule => Ok(Cow::Borrowed(object)),
        Err(unheld) => Err(Failure::RefusedWithoutAVerdict(refusal(
            &unheld, rule, &m.oid,
        ))),
    }
}

fn refusal(unheld: &Unheld, rule: &str, oid: &str) -> String {
    let (why, held) = match unheld {
        Unheld::Interval { days } => (
            "Todoist cannot combine a monthly interval with a weekday of the month",
            format!("every month on {days}"),
        ),
        Unheld::CountedBack { weekday } => (
            "Todoist counts back from the end of a month only to its last weekday",
            format!("every month on -1{weekday}"),
        ),
    };
    format!(
        "{why}, so it cannot hold {rule:?}; nothing was sent for this task; \
         give it a rule Todoist can hold, such as: dam edit {oid} --recurrence {held:?}"
    )
}
