use super::Object;
use crate::{Oid, Path};

pub fn nearest_parents<'a>(path: &Path, objects: impl IntoIterator<Item = &'a Object>) -> Vec<Oid> {
    let holders: Vec<(usize, &Oid)> = objects
        .into_iter()
        .filter_map(|o| {
            let folder = o.base().owned_folder()?;
            path.is_within(&folder).then(|| (folder.depth(), o.oid()))
        })
        .collect();
    let nearest = holders.iter().map(|(depth, _)| *depth).max();
    holders
        .into_iter()
        .filter(|(depth, _)| Some(*depth) == nearest)
        .map(|(_, oid)| oid.clone())
        .collect()
}

#[cfg(test)]
mod tests;
