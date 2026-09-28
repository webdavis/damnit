pub fn credential_variable(remote: &str, name: &str) -> String {
    let upper_snake = |s: &str| {
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
    format!("DAM_{}_{}", upper_snake(remote), upper_snake(name))
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
