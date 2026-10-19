use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    pub name: String,
    pub values: Vec<String>,
    pub exclusive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Categories(Vec<Category>);

#[derive(Debug, PartialEq, Eq)]
pub enum CategoryError {
    DuplicateValue {
        value: String,
        first: String,
        second: String,
    },
    EmptyCategory(String),
    DuplicateName(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum LabelViolation {
    Exclusive { category: String, held: Vec<String> },
}

impl Categories {
    pub fn new(categories: Vec<Category>) -> Result<Categories, CategoryError> {
        let mut names = BTreeSet::new();
        let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
        for category in &categories {
            if category.values.is_empty() {
                return Err(CategoryError::EmptyCategory(category.name.clone()));
            }
            if !names.insert(category.name.as_str()) {
                return Err(CategoryError::DuplicateName(category.name.clone()));
            }
            for value in &category.values {
                if let Some(first) = owner.insert(value, &category.name) {
                    return Err(CategoryError::DuplicateValue {
                        value: value.clone(),
                        first: first.to_string(),
                        second: category.name.clone(),
                    });
                }
            }
        }
        Ok(Categories(categories))
    }

    pub fn check(&self, labels: &BTreeSet<String>) -> Result<(), LabelViolation> {
        for category in self.0.iter().filter(|c| c.exclusive) {
            let held: Vec<String> = labels
                .iter()
                .filter(|l| category.values.contains(l))
                .cloned()
                .collect();
            if held.len() > 1 {
                return Err(LabelViolation::Exclusive {
                    category: category.name.clone(),
                    held,
                });
            }
        }
        Ok(())
    }

    pub fn category_of(&self, value: &str) -> Option<&Category> {
        self.0.iter().find(|c| c.values.iter().any(|v| v == value))
    }

    pub fn category_of_name(&self, name: &str) -> Option<&Category> {
        self.0.iter().find(|c| c.name == name)
    }

    pub fn is_free(&self, value: &str) -> bool {
        self.category_of(value).is_none()
    }

    pub fn all(&self) -> &[Category] {
        &self.0
    }
}

impl fmt::Display for CategoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CategoryError::DuplicateValue {
                value,
                first,
                second,
            } => {
                write!(
                    f,
                    "label {value:?} is declared in both {first:?} and {second:?}"
                )
            }
            CategoryError::EmptyCategory(n) => write!(f, "category {n:?} declares no values"),
            CategoryError::DuplicateName(n) => write!(f, "category {n:?} is declared twice"),
        }
    }
}

impl fmt::Display for LabelViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelViolation::Exclusive { category, held } => {
                write!(
                    f,
                    "category {category:?} allows one value, got {}",
                    held.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for CategoryError {}
impl std::error::Error for LabelViolation {}

#[cfg(test)]
mod tests;
