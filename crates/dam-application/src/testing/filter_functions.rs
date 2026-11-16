use std::cell::RefCell;

use dam_domain::WorkedOut;

use crate::ports::{FilterFunctions, FunctionFailure, Handed, Histories, History};

#[derive(Default)]
pub struct FakeFilterFunctions {
    pub keeping: Vec<&'static str>,
    pub never_reading_history: bool,
    pub failing: RefCell<Option<FunctionFailure>>,
    pub failing_on: Option<(&'static str, FunctionFailure)>,
    pub seen: RefCell<Vec<(String, Option<History>)>>,
    pub handed: RefCell<Vec<(String, WorkedOut)>>,
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
        handed: &[Handed],
        histories: &dyn Histories,
    ) -> Result<Vec<bool>, FunctionFailure> {
        self.handed.borrow_mut().extend(
            handed
                .iter()
                .map(|h| (h.object.base().subject.clone(), h.worked_out.clone())),
        );
        let objects: Vec<_> = handed.iter().map(|h| &h.object).collect();
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
        if let Some((subject, failure)) = &self.failing_on
            && objects.iter().any(|o| o.base().subject == *subject)
        {
            return Err(failure.clone());
        }
        Ok(objects
            .iter()
            .map(|o| self.keeping.contains(&o.base().subject.as_str()))
            .collect())
    }
}
