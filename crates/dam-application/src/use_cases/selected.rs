use dam_domain::{Command, Filter, Object, Order, Path, Scope, WorkedOut, scope, worked_out};

use crate::config::FilterConfig;
use crate::errors::UseCaseError;
use crate::service::Dam;

mod error;
mod event_days;
mod facts;
mod pulled;
mod saved;

pub use error::SelectError;
use facts::Facts;
pub use pulled::BeyondPulled;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Defaults {
    Of(Command),
    Fixed(Scope),
}

pub struct Listed {
    pub objects: Vec<Object>,
    pub worked_out: Vec<WorkedOut>,
}

impl<'a> Dam<'a> {
    pub fn list_selected(
        &self,
        filter: &Filter,
        order: &Order,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<Vec<Object>, UseCaseError> {
        Ok(self
            .list_selected_noting(filter, order, place, local)?
            .0
            .objects)
    }

    pub fn list_selected_noting(
        &self,
        filter: &Filter,
        order: &Order,
        place: &Path,
        local: &[FilterConfig],
    ) -> Result<(Listed, Option<BeyondPulled>), UseCaseError> {
        let (listed, facts) = self.selected(filter, Command::List, place, local)?;
        let beyond = facts.beyond_pulled(filter);
        let objects = order.arrange(listed, &facts);
        let worked_out = objects.iter().map(|o| worked_out(o, &facts)).collect();
        let listed = Listed {
            objects,
            worked_out,
        };
        Ok((listed, beyond))
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

    pub(crate) fn chosen(
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
        let mut facts = self.facts(categories, &everything, saved)?;
        let candidates: Vec<Object> = everything
            .into_iter()
            .filter(|o| o.base().path.is_within(place) && scope.admits(o))
            .collect();
        let (listed, rest): (Vec<Object>, Vec<Object>) = candidates
            .into_iter()
            .partition(|o| filter.matches(o, &facts));
        facts.count_unplaced(filter, &rest);
        Ok((listed, facts))
    }

    pub(crate) fn scope_once(
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
mod children_tests;
#[cfg(test)]
mod defaults_once_tests;
#[cfg(test)]
mod event_days_tests;
#[cfg(test)]
mod from_object_tests;
#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod saved_tests;
#[cfg(test)]
pub(crate) mod tests;
