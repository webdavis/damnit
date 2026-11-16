use dam_application::SelectError;
use dam_domain::Problem;

pub(super) fn sorted(error: &SelectError, sort: &[String]) -> Vec<String> {
    let SelectError::Sort(e) = error else {
        return sort.to_vec();
    };
    let replacement = match &e.problem {
        Problem::UnknownField { closest: Some(c) } => match e.value.split_once(':') {
            Some((_, direction)) => Some(format!("{c}:{direction}")),
            None => Some(c.to_string()),
        },
        Problem::UnknownField { closest: None } => None,
        _ => Some(e.field.clone()),
    };
    sort.iter()
        .filter_map(|text| {
            let parts: Vec<String> = text
                .split(',')
                .filter_map(|part| match part == e.value {
                    true => replacement.clone(),
                    false => Some(part.to_string()),
                })
                .collect();
            Some(parts.join(",")).filter(|joined| !joined.is_empty())
        })
        .collect()
}
