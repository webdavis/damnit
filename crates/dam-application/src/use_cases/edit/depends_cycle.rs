use std::collections::BTreeMap;

use dam_domain::{Oid, cycle_in};

use crate::errors::UseCaseError;
use crate::ports::ObjectRepository;
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn cycle_through(
        &self,
        oid: &Oid,
        added: &[Oid],
    ) -> Result<Option<Vec<Oid>>, UseCaseError> {
        let edges = depends_reachable_from(self.store, added)?;
        let edges_of = |o: &Oid| edges.get(o).cloned().unwrap_or_default();
        Ok(cycle_in(added, oid, &edges_of))
    }
}

fn depends_reachable_from(
    objects: &dyn ObjectRepository,
    starts: &[Oid],
) -> Result<BTreeMap<Oid, Vec<Oid>>, UseCaseError> {
    let mut edges = BTreeMap::new();
    let mut unvisited = starts.to_vec();
    while let Some(at) = unvisited.pop() {
        if edges.contains_key(&at) {
            continue;
        }
        let depends = objects
            .get(&at)?
            .map(|o| o.base().depends.clone())
            .unwrap_or_default();
        unvisited.extend(depends.iter().cloned());
        edges.insert(at, depends);
    }
    Ok(edges)
}
