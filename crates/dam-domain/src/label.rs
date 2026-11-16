const RESERVED: [char; 6] = ['&', '|', '!', '(', ')', '"'];

pub fn names_a_label(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || RESERVED.contains(&c))
}

#[cfg(test)]
mod tests;
