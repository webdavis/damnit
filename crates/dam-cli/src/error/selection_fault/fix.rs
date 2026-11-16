use dam_application::SelectError;
use dam_domain::{DefaultsError, FieldType, FilterTable, Problem, TableValue, select_field};

pub(super) fn dropped_or_moved(error: &DefaultsError) -> &'static str {
    match error {
        DefaultsError::KindContradicts { .. } => "kind",
        DefaultsError::DoneContradicts { .. } => "done",
        DefaultsError::BothKinds { event_field, .. } => event_field,
        DefaultsError::TaskFieldInAgenda { field } => field,
    }
}

pub(super) fn typed_text(typed: &FilterTable, key: &str) -> String {
    let top = typed.fields.get(key).map(|v| v.texts().join(","));
    let any = || typed.any.iter().find_map(|t| t.fields.get(key));
    top.or_else(|| any().map(|v| v.texts().join(",")))
        .unwrap_or_default()
}

pub(super) fn corrected(error: &SelectError, typed: &FilterTable) -> FilterTable {
    let mut t = typed.clone();
    match error {
        SelectError::Value(e) => match &e.problem {
            Problem::UnknownField { closest: Some(c) } => rename(&mut t, &e.field, c),
            Problem::UnknownField { closest: None } => remove(&mut t, &e.field),
            Problem::BadValue => replace(&mut t, &e.field, &e.value, &example(&e.field)),
            Problem::TwoAtOnce { fix: Some(list) } => {
                t.fields
                    .insert(e.field.clone(), TableValue::One(list.clone()));
            }
            Problem::TwoAtOnce { fix: None } => move_to_any(&mut t, &e.field),
            Problem::NotAGroup { groups } => match groups.first() {
                Some(group) => replace(&mut t, &e.field, &e.value, group),
                None => remove(&mut t, &e.field),
            },
            Problem::NotInGroup { group, labels } => match labels.first() {
                Some(label) => replace(&mut t, &e.field, &e.value, &format!("{group}={label}")),
                None => replace(&mut t, &e.field, &e.value, group),
            },
        },
        SelectError::Defaults(DefaultsError::BothKinds {
            task_field,
            event_field,
        }) => {
            move_to_any(&mut t, task_field);
            move_to_any(&mut t, event_field);
        }
        SelectError::Defaults(DefaultsError::TaskFieldInAgenda { .. }) | SelectError::Sort(_) => {}
        SelectError::Defaults(e) => remove(&mut t, dropped_or_moved(e)),
    }
    t
}

fn example(key: &str) -> String {
    let Some(field) = select_field(key) else {
        return String::new();
    };
    match field.ty {
        FieldType::Date => "today".into(),
        FieldType::Number(lo, _) => lo.to_string(),
        _ => field.shapes().into_iter().next().unwrap_or_default(),
    }
}

fn replace(t: &mut FilterTable, key: &str, from: &str, to: &str) {
    if let Some(value) = t.fields.get_mut(key) {
        *value = value.map(|text| if text == from { to } else { text }.to_string());
    }
    t.any.iter_mut().for_each(|a| replace(a, key, from, to));
}

fn rename(t: &mut FilterTable, from: &str, to: &str) {
    for entry in &mut t.any {
        if let Some(value) = entry.fields.remove(from) {
            entry.fields.insert(to.into(), value);
        }
    }
}

fn remove(t: &mut FilterTable, key: &str) {
    t.fields.remove(key);
    t.any.iter_mut().for_each(|a| remove(a, key));
    t.any.retain(|a| !(a.fields.is_empty() && a.any.is_empty()));
}

fn move_to_any(t: &mut FilterTable, key: &str) {
    let Some(value) = t.fields.remove(key) else {
        return;
    };
    t.any.extend(value.texts().iter().map(|text| FilterTable {
        fields: [(key.to_string(), TableValue::One(text.clone()))].into(),
        any: Vec::new(),
    }));
}
