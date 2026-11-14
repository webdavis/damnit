#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterConfig {
    pub name: String,
    pub rule: FilterRule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterRule {
    Query(String),
    Function,
    QueryThenFunction(String),
}

impl FilterRule {
    pub fn query(&self) -> Option<&str> {
        match self {
            FilterRule::Query(text) | FilterRule::QueryThenFunction(text) => Some(text),
            FilterRule::Function => None,
        }
    }

    pub fn runs_a_function(&self) -> bool {
        !matches!(self, FilterRule::Query(_))
    }
}
