use dam_protocol::WireObject;

pub struct Carries {
    fields: &'static [&'static str],
    sentence: &'static str,
}

pub const PROJECT: Carries = Carries {
    fields: &["subject", "done"],
    sentence: "a Todoist project carries only its name and whether it is archived",
};

pub const SECTION: Carries = Carries {
    fields: &["subject"],
    sentence: "a Todoist section carries only its name",
};

pub fn fields_the_container_cannot_carry(
    carries: &Carries,
    declared: &[String],
    object: &WireObject,
    held: Option<&WireObject>,
) -> Option<String> {
    let stuck: Vec<&str> = declared
        .iter()
        .map(String::as_str)
        .filter(|f| !carries.fields.contains(f))
        .filter(|f| held.is_none_or(|h| differs(f, object, h)))
        .collect();
    if stuck.is_empty() {
        return None;
    }
    Some(format!(
        "{}; {} did not change",
        carries.sentence,
        stuck.join(", ")
    ))
}

fn differs(field: &str, declared: &WireObject, held: &WireObject) -> bool {
    let (d, h) = (declared.task.as_ref(), held.task.as_ref());
    match field {
        "path" => declared.path != held.path,
        "body" => declared.body != held.body,
        "labels" => declared.labels != held.labels,
        "recurrence" => declared.recurrence != held.recurrence,
        "done" => d.map(|t| t.done) != h.map(|t| t.done),
        "priority" => d.map(|t| t.priority) != h.map(|t| t.priority),
        "due" => d.and_then(|t| t.due.as_deref()) != h.and_then(|t| t.due.as_deref()),
        "deadline" => {
            d.and_then(|t| t.deadline.as_deref()) != h.and_then(|t| t.deadline.as_deref())
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests;
