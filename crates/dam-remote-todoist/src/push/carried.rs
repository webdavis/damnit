use dam_protocol::WireObject;

struct Carries {
    fields: &'static [&'static str],
    sentence: &'static str,
}

const PROJECT: Carries = Carries {
    fields: &["subject", "done"],
    sentence: "a Todoist project carries only its name and whether it is archived",
};

const SECTION: Carries = Carries {
    fields: &["subject"],
    sentence: "a Todoist section carries only its name",
};

pub fn fields_the_container_cannot_carry(
    kind: char,
    declared: &[String],
    object: &WireObject,
    held: Option<&WireObject>,
) -> Option<String> {
    let carries = if kind == 'p' { PROJECT } else { SECTION };
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
mod tests {
    use super::*;
    use dam_protocol::WireTask;

    fn object(path: &str, done: bool) -> WireObject {
        WireObject {
            oid: String::new(),
            remote_id: None,
            kind: "task".into(),
            subject: "Now".into(),
            body: String::new(),
            path: path.into(),
            labels: vec![],
            depends: vec![],
            reminders: vec![],
            recurrence: None,
            task: Some(WireTask {
                done,
                completed_at: None,
                priority: 4,
                due: None,
                deadline: None,
                event: None,
            }),
            event: None,
        }
    }

    #[test]
    fn names_every_field_the_container_cannot_carry_and_nothing_else() {
        let declared = ["subject".to_string(), "done".into(), "body".into()];
        let (object, held) = (object("Work/", true), object("Work/", false));
        assert_eq!(
            fields_the_container_cannot_carry('p', &declared[..2], &object, Some(&held)),
            None
        );
        assert_eq!(
            fields_the_container_cannot_carry('s', &declared, &object, Some(&held)).as_deref(),
            Some("a Todoist section carries only its name; done did not change")
        );
    }

    #[test]
    fn without_a_held_object_every_uncarried_field_counts_as_changed() {
        let declared = ["path".to_string(), "done".into()];
        let object = object("Work/", false);
        assert_eq!(
            fields_the_container_cannot_carry('s', &declared, &object, None).as_deref(),
            Some("a Todoist section carries only its name; path, done did not change")
        );
    }
}
