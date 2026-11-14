use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Priority(u8);

#[derive(Debug, PartialEq, Eq)]
pub struct PriorityError(pub u8);

impl Priority {
    pub const HIGHEST: Priority = Priority(1);
    pub const LOWEST: Priority = Priority(4);

    pub fn new(value: u8) -> Result<Priority, PriorityError> {
        if (1..=4).contains(&value) {
            Ok(Priority(value))
        } else {
            Err(PriorityError(value))
        }
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Priority {
        Priority::LOWEST
    }
}

impl fmt::Display for PriorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "priority is 1 to 4, got {}", self.0)
    }
}

impl std::error::Error for PriorityError {}

#[cfg(test)]
mod tests;
