use dam_domain::{Object, Path};

use crate::config::FilterConfig;
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn list_within(&self, place: &Path) -> Result<Vec<Object>, UseCaseError> {
        self.seen_within(place)
    }

    pub fn list_saved(&self, name: &str) -> Result<Vec<Object>, UseCaseError> {
        self.listed_by_name(name, &Path::root(), &[])
    }

    pub(crate) fn kept(
        &self,
        filter: &FilterConfig,
        out: Vec<Object>,
    ) -> Result<Vec<Object>, UseCaseError> {
        match filter.runs_a_function {
            true => self.kept_by_function(&filter.name, out),
            false => Ok(out),
        }
    }
}

mod by_function;
pub(crate) mod log_by_object;
mod order;
mod seen;
pub(crate) mod trial;

#[cfg(test)]
mod access_tests;

#[cfg(test)]
mod function_tests;
#[cfg(test)]
mod function_worked_tests;

#[cfg(test)]
mod history_reads_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod trial_tests;

#[cfg(test)]
mod within_tests;
