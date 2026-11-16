use std::collections::BTreeSet;

use dam_domain::Categories;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelMap(Vec<(String, String)>);

#[derive(Debug, PartialEq, Eq)]
pub enum MapProblem {
    Empty,
    Blank,
    Twice(String),
    Same(String),
    TwoCategories {
        old: String,
        new: String,
        old_category: String,
        new_category: String,
    },
}

impl LabelMap {
    pub fn new(
        pairs: Vec<(String, String)>,
        categories: &Categories,
    ) -> Result<LabelMap, MapProblem> {
        if pairs.is_empty() {
            return Err(MapProblem::Empty);
        }
        let mut olds = BTreeSet::new();
        for (old, new) in &pairs {
            if old.is_empty() || new.is_empty() {
                return Err(MapProblem::Blank);
            }
            if old == new {
                return Err(MapProblem::Same(old.clone()));
            }
            if !olds.insert(old) {
                return Err(MapProblem::Twice(old.clone()));
            }
            if let (Some(from), Some(to)) =
                (categories.category_of(old), categories.category_of(new))
                && from.name != to.name
            {
                return Err(MapProblem::TwoCategories {
                    old: old.clone(),
                    new: new.clone(),
                    old_category: from.name.clone(),
                    new_category: to.name.clone(),
                });
            }
        }
        Ok(LabelMap(pairs))
    }

    pub fn pairs(&self) -> &[(String, String)] {
        &self.0
    }

    pub fn new_of(&self, label: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(old, _)| old == label)
            .map(|(_, new)| new.as_str())
    }

    pub(super) fn touches(&self, labels: &BTreeSet<String>) -> bool {
        labels.iter().any(|label| self.new_of(label).is_some())
    }

    pub(super) fn applied(&self, labels: &BTreeSet<String>) -> BTreeSet<String> {
        labels
            .iter()
            .map(|label| self.new_of(label).unwrap_or(label).to_string())
            .collect()
    }

    pub(super) fn swaps(&self, labels: &BTreeSet<String>) -> (Vec<String>, Vec<String>) {
        labels
            .iter()
            .filter_map(|label| Some((label.clone(), self.new_of(label)?.to_string())))
            .unzip()
    }
}
