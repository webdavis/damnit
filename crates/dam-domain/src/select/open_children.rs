use std::collections::BTreeMap;

use crate::{Oid, Task};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenChildren(BTreeMap<Oid, usize>);

impl OpenChildren {
    pub fn of<'a>(tasks: impl IntoIterator<Item = &'a Task>) -> OpenChildren {
        let mut counts = BTreeMap::new();
        for task in tasks {
            if let Some(parent) = &task.base.parent
                && !task.completion.is_done()
            {
                *counts.entry(parent.clone()).or_default() += 1;
            }
        }
        OpenChildren(counts)
    }

    pub fn of_parent(&self, parent: &Oid) -> usize {
        self.0.get(parent).copied().unwrap_or(0)
    }
}
