use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A named set of label values. `exclusive` means an object holds at most one
/// of them. dam ships none; users declare their own.
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
mod tests {
    use super::*;

    fn effort() -> Category {
        Category {
            name: "effort".into(),
            values: vec!["light".into(), "admin".into(), "deep".into()],
            exclusive: true,
        }
    }

    fn context() -> Category {
        Category {
            name: "context".into(),
            values: vec!["home".into(), "office".into()],
            exclusive: false,
        }
    }

    fn labels(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_value_in_two_categories_is_refused_at_definition() {
        let clash = Category {
            name: "mode".into(),
            values: vec!["deep".into()],
            exclusive: false,
        };
        let err = Categories::new(vec![effort(), clash]).unwrap_err();
        assert_eq!(
            err,
            CategoryError::DuplicateValue {
                value: "deep".into(),
                first: "effort".into(),
                second: "mode".into()
            }
        );
    }

    #[test]
    fn an_empty_category_and_a_duplicate_name_are_refused() {
        let empty = Category {
            name: "x".into(),
            values: vec![],
            exclusive: true,
        };
        assert_eq!(
            Categories::new(vec![empty]).unwrap_err(),
            CategoryError::EmptyCategory("x".into())
        );
        assert_eq!(
            Categories::new(vec![effort(), effort()]).unwrap_err(),
            CategoryError::DuplicateName("effort".into())
        );
    }

    #[test]
    fn one_value_from_an_exclusive_category_plus_free_labels_is_fine() {
        let cats = Categories::new(vec![effort(), context()]).unwrap();
        assert_eq!(cats.check(&labels(&["deep", "errand", "waiting"])), Ok(()));
    }

    #[test]
    fn two_values_from_an_exclusive_category_are_refused() {
        let cats = Categories::new(vec![effort()]).unwrap();
        let err = cats
            .check(&labels(&["deep", "light", "errand"]))
            .unwrap_err();
        assert_eq!(
            err,
            LabelViolation::Exclusive {
                category: "effort".into(),
                held: vec!["deep".into(), "light".into()]
            }
        );
    }

    #[test]
    fn several_values_from_a_multiple_category_are_fine() {
        let cats = Categories::new(vec![context()]).unwrap();
        assert_eq!(cats.check(&labels(&["home", "office"])), Ok(()));
    }

    #[test]
    fn category_of_and_is_free() {
        let cats = Categories::new(vec![effort()]).unwrap();
        assert_eq!(
            cats.category_of("deep").map(|c| c.name.as_str()),
            Some("effort")
        );
        assert!(cats.is_free("errand"));
        assert!(!cats.is_free("deep"));
    }

    #[test]
    fn category_of_name_finds_by_declared_name() {
        let cats = Categories::new(vec![effort()]).unwrap();
        assert!(cats.category_of_name("effort").is_some());
        assert!(cats.category_of_name("mood").is_none());
    }

    #[test]
    fn no_categories_means_every_label_is_free() {
        let cats = Categories::new(vec![]).unwrap();
        assert_eq!(cats.check(&labels(&["a", "b"])), Ok(()));
    }
}
