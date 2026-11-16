use dam_domain::{Object, Path};

use super::order::order;
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn seen_within(&self, place: &Path) -> Result<Vec<Object>, UseCaseError> {
        let today = self.clock.today();
        let mut seen = self
            .store
            .all()?
            .into_iter()
            .filter(|o| self.access.sees(o) && o.base().path.is_within(place))
            .map(|o| self.as_seen(o))
            .collect::<Result<Vec<Object>, _>>()?;
        seen.sort_by(|a, b| order(a, b, today));
        Ok(seen)
    }
}
