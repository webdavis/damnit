use dam_domain::{Command, Filter, Object, Order, Path, scope};

use crate::config::FilterConfig;
use crate::errors::UseCaseError;
use crate::service::Dam;

mod error;
mod event_days;
mod facts;

use facts::Facts;
mod saved;

pub use error::SelectError;

impl<'a> Dam<'a> {
    pub fn list_selected(
        &self,
        filter: &Filter,
        order: &Order,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<Vec<Object>, UseCaseError> {
        let (listed, facts) = self.selected(filter, Command::List, place, local)?;
        Ok(order.arrange(listed, &facts))
    }

    pub(crate) fn selected(
        &self,
        filter: &Filter,
        command: Command,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<(Vec<Object>, Facts<'a>), UseCaseError> {
        let categories = self.config.categories_at(place);
        filter.check(categories).map_err(SelectError::Value)?;
        let scope = scope(filter, command).map_err(SelectError::Defaults)?;
        let everything = self.seen_within(&Path::root())?;
        let saved = self.saved(filter, place, local)?;
        let facts = self.facts(categories, &everything, saved)?;
        let listed = everything
            .into_iter()
            .filter(|o| o.base().path.is_within(place))
            .filter(|o| scope.admits(o) && filter.matches(o, &facts))
            .collect();
        Ok((listed, facts))
    }
}

#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod event_days_tests;
#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod tests;
