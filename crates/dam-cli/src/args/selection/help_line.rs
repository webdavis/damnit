use dam_domain::{FieldType, Has, Holds, SelectField};

pub(super) fn heading(field: &SelectField) -> &'static str {
    match field.has {
        Has::Both => "Which",
        Has::Task => "Tasks",
        Has::Event => "Events",
    }
}

pub(super) fn is_yes_no(field: &SelectField) -> bool {
    matches!(field.ty, FieldType::YesNo)
}

pub(super) fn value_name(field: &SelectField) -> &'static str {
    match field.ty {
        FieldType::Text => "TEXT",
        FieldType::Choice(_) => "CHOICE",
        FieldType::Number(..) => "N",
        FieldType::Length => "LENGTH",
        FieldType::TextSet => "VALUE",
        FieldType::Path => "FOLDER",
        FieldType::Reference => "ID",
        FieldType::YesNo => "BOOL",
        FieldType::Date => "DATE",
        FieldType::Category => "GROUP[=LABEL]",
        FieldType::FilterName => "FILTER",
    }
}

pub(super) fn help(field: &SelectField) -> String {
    let line = match field.ty {
        FieldType::Text => "Holds this text, ignoring case".to_string(),
        FieldType::Choice(choices) => format!("One of {}; a,b for either", choices().join(", ")),
        FieldType::Number(lo, hi) => format!("A number from {lo} to {hi}, a list 1,2 or a span ..2"),
        FieldType::Length => "A length or a span: 30m, 30m..2h, ..5m".to_string(),
        FieldType::TextSet => "Exactly this value; a,b for either".to_string(),
        FieldType::Path => "At or under this folder; a/,b/ for either".to_string(),
        FieldType::Reference => "This id, a list of ids, or in:FILTER".to_string(),
        FieldType::YesNo => "true, false or any; the bare flag means true".to_string(),
        FieldType::Date => {
            "A day, period, anchor, span or repeating day: 2026-11-20, 2026-W47, today+3, ..today, on:thu"
                .to_string()
        }
        FieldType::Category => "A label group, or GROUP=LABEL; GROUP=a,b for either".to_string(),
        FieldType::FilterName => "Listed by this saved filter; a,b for either".to_string(),
    };
    match field.holds {
        Holds::One => line,
        _ => format!("{line}; none for empty"),
    }
}
