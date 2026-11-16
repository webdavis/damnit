use dam_domain::{Command, Filter, Object, Order, Path, Scope, scope};

use crate::config::FilterConfig;
use crate::errors::UseCaseError;
use crate::service::Dam;

mod error;
mod event_days;
mod facts;
mod saved;

pub use error::SelectError;
use facts::Facts;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Defaults {
    Of(Command),
    Fixed(Scope),
}

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
        self.chosen(filter, Defaults::Of(command), place, local, &[])
    }

    fn chosen(
        &self,
        filter: &Filter,
        defaults: Defaults,
        place: &Path,
        local: &[FilterConfig],
        chain: &[String],
    ) -> Result<(Vec<Object>, Facts<'a>), UseCaseError> {
        let categories = self.config.categories_at(place);
        filter.check(categories).map_err(SelectError::Value)?;
        let scope = match defaults {
            Defaults::Fixed(scope) => scope,
            Defaults::Of(command) => self.scope_once(filter, command, place, local)?,
        };
        let everything = self.seen_within(&Path::root())?;
        let saved = self.saved(filter, (place, local), chain)?;
        let facts = self.facts(categories, &everything, saved)?;
        let listed = everything
            .into_iter()
            .filter(|o| o.base().path.is_within(place))
            .filter(|o| scope.admits(o) && filter.matches(o, &facts))
            .collect();
        Ok((listed, facts))
    }

    fn scope_once(
        &self,
        filter: &Filter,
        command: Command,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<Scope, UseCaseError> {
        let lent = |name: &str| {
            let (config, _, _) = self.saved_config(name, place, local).ok()?;
            Some(config.table.clone())
        };
        let view = Filter::read(&filter.write().with_saved(&lent)).map_err(SelectError::Value)?;
        Ok(scope(&view, command).map_err(SelectError::Defaults)?)
    }
}

#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod defaults_once_tests;
#[cfg(test)]
mod event_days_tests;
#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod saved_tests;
#[cfg(test)]
pub(crate) mod tests;
