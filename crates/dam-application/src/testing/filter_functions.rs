use std::cell::RefCell;

use dam_domain::Object;

use crate::ports::{FilterFailure, FilterFunctions, Histories, History};

#[derive(Default)]
pub struct FakeFilterFunctions {
    pub keeping: Vec<&'static str>,
    pub never_reading_history: bool,
    pub failing: RefCell<Option<FilterFailure>>,
    pub seen: RefCell<Vec<(String, Option<History>)>>,
}

impl FakeFilterFunctions {
    pub fn keeping(subjects: &[&'static str]) -> FakeFilterFunctions {
        FakeFilterFunctions {
            keeping: subjects.to_vec(),
            ..FakeFilterFunctions::default()
        }
    }

    pub fn seen_subjects(&self) -> Vec<String> {
        self.seen.borrow().iter().map(|(s, _)| s.clone()).collect()
    }
}

impl FilterFunctions for FakeFilterFunctions {
    fn keeps(
        &self,
        _filter: &str,
        objects: &[Object],
        histories: &dyn Histories,
    ) -> Result<Vec<bool>, FilterFailure> {
        let seen: Vec<_> = objects
            .iter()
            .map(|o| {
                let history = (!self.never_reading_history).then(|| histories.of(o.oid()));
                (o.base().subject.clone(), history)
            })
            .collect();
        self.seen.borrow_mut().extend(seen);
        if let Some(failure) = self.failing.borrow_mut().take() {
            return Err(failure);
        }
        Ok(objects
            .iter()
            .map(|o| self.keeping.contains(&o.base().subject.as_str()))
            .collect())
    }
}
