use super::fields::select_field;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterError {
    pub field: String,
    pub value: String,
    pub problem: Problem,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    UnknownField { closest: Option<&'static str> },
    BadValue,
    TwoAtOnce { fix: Option<String> },
    NotAGroup { groups: Vec<String> },
    NotInGroup { group: String, labels: Vec<String> },
}

impl FilterError {
    pub fn shapes(&self) -> Vec<String> {
        select_field(&self.field).map_or_else(Vec::new, |f| f.shapes())
    }
}
