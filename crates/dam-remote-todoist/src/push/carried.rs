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

pub fn fields_the_container_cannot_carry(kind: char, changed: &[String]) -> Option<String> {
    let carries = if kind == 'p' { PROJECT } else { SECTION };
    let stuck: Vec<&str> = changed
        .iter()
        .map(String::as_str)
        .filter(|f| !carries.fields.contains(f))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_every_field_the_container_cannot_carry_and_nothing_else() {
        let changed = ["subject".to_string(), "done".into(), "body".into()];
        assert_eq!(fields_the_container_cannot_carry('p', &changed[..2]), None);
        assert_eq!(
            fields_the_container_cannot_carry('s', &changed).as_deref(),
            Some("a Todoist section carries only its name; done, body did not change")
        );
    }
}
