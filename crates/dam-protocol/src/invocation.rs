//! How dam names what it hands a helper.

/// Where a helper reads one credential: `DAM_<REMOTE>_<NAME>`, uppercased with
/// every non-alphanumeric character folded to `_`.
pub fn credential_variable(remote: &str, name: &str) -> String {
    let shout = |s: &str| {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>()
    };
    format!("DAM_{}_{}", shout(remote), shout(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_variables_are_upper_snake() {
        assert_eq!(
            credential_variable("todoist", "api_token"),
            "DAM_TODOIST_API_TOKEN"
        );
        assert_eq!(
            credential_variable("my-cal", "client-id"),
            "DAM_MY_CAL_CLIENT_ID"
        );
    }
}
