#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    New,
    Changed,
    Duplicate,
}

impl Answer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Answer::New => "new",
            Answer::Changed => "changed",
            Answer::Duplicate => "duplicate",
        }
    }
}
