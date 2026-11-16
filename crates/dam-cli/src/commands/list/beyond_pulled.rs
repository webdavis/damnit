use dam_application::BeyondPulled;

use crate::args::selection::flag_of;

pub(super) fn warning(beyond: &BeyondPulled) -> String {
    let count = beyond.tasks + beyond.events;
    let noun = match (beyond.tasks, beyond.events) {
        (_, 0) => "task",
        (0, _) => "event",
        _ => "object",
    };
    let (counted, verb, them) = match count {
        1 => (format!("1 {noun}"), "is", "it"),
        n => (format!("{n} {noun}s"), "are", "them"),
    };
    let field = beyond.fields.first().copied().unwrap_or("due");
    let flag = flag_of(field);
    let dated = match field {
        "due" => "due".to_string(),
        _ => format!("dated by --{flag}"),
    };
    let names: Vec<String> = beyond.filters.iter().map(|f| format!("on:{f}")).collect();
    let names = names.join(" or ");
    match beyond.held {
        Some((first, last)) => {
            let before = first.yesterday().unwrap_or(first);
            let after = last.tomorrow().unwrap_or(last);
            format!(
                "dam: warning: {counted} {verb} {dated} outside the days dam holds events for ({first} to {last}), so {names} could not place {them}; dam list --any {flag}=..{before} --any {flag}={after}.. lists {them}"
            )
        }
        None => format!(
            "dam: warning: dam holds no events, so {names} could not place {counted} that {verb} {dated}; dam list --{flag} not:none lists {them}"
        ),
    }
}
