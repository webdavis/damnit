use std::collections::BTreeMap;

use crate::Object;

pub fn labels_in_use(objects: &[Object]) -> BTreeMap<String, usize> {
    let mut counted = BTreeMap::new();
    for label in objects.iter().flat_map(|o| &o.base().labels) {
        *counted.entry(label.clone()).or_default() += 1;
    }
    counted
}

#[cfg(test)]
mod tests;
