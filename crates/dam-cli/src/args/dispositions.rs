use dam_domain::{ChildDisposition, DependencyDisposition};

pub(super) fn child_disposition(text: &str) -> Result<ChildDisposition, String> {
    match text.split_once(':') {
        Some(("into", name)) if !name.trim().is_empty() => {
            Ok(ChildDisposition::Into(name.trim().to_string()))
        }
        _ => match text {
            "up" => Ok(ChildDisposition::Up),
            "keep" => Ok(ChildDisposition::Keep),
            _ => Err("one of up, keep, into:<name>".to_string()),
        },
    }
}

pub(super) fn dependency_disposition(text: &str) -> Result<DependencyDisposition, String> {
    match text {
        "drop" => Ok(DependencyDisposition::Drop),
        "keep" => Ok(DependencyDisposition::Keep),
        _ => Err("one of drop, keep".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_name_is_taken_without_the_space_around_it() {
        assert_eq!(
            child_disposition("into:  leftovers  "),
            Ok(ChildDisposition::Into("leftovers".into()))
        );
    }

    #[test]
    fn a_blank_group_name_is_refused_since_it_would_name_a_whitespace_group_task() {
        for blank in ["into:", "into: ", "into:\t", "into:   "] {
            assert!(child_disposition(blank).is_err(), "{blank:?}");
        }
    }
}
