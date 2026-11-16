use std::collections::BTreeMap;

use crate::{Object, Path};

pub fn labels_in_use(objects: &[Object]) -> BTreeMap<String, usize> {
    let mut counted = BTreeMap::new();
    for label in objects.iter().flat_map(|o| &o.base().labels) {
        *counted.entry(label.clone()).or_default() += 1;
    }
    counted
}

pub fn paths_in_use(objects: &[Object], place: &Path) -> BTreeMap<Path, usize> {
    let mut counted = BTreeMap::new();
    for object in objects {
        let at = Some(object.base().path.clone());
        for path in std::iter::successors(at, Path::parent)
            .take_while(|p| *p != Path::root() && p.is_within(place))
        {
            *counted.entry(path).or_default() += 1;
        }
    }
    counted
}

#[cfg(test)]
mod tests;
