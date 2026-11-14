use std::collections::BTreeSet;

use crate::Oid;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Blockers {
    pub dependencies: Vec<Oid>,
    pub children: Vec<Oid>,
}

impl Blockers {
    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty() && self.children.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tracked {
    No,
    ByRemote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Force {
    No,
    With(Dispositions),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dispositions {
    pub children: ChildDisposition,
    pub dependencies: DependencyDisposition,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ChildDisposition {
    Up,
    Into(String),
    #[default]
    Keep,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DependencyDisposition {
    Drop,
    #[default]
    Keep,
}

pub fn cycle_in(
    depends: &[Oid],
    oid: &Oid,
    existing_depends_of: &dyn Fn(&Oid) -> Vec<Oid>,
) -> Option<Vec<Oid>> {
    for start in depends {
        let mut path = vec![start.clone()];
        let mut seen = BTreeSet::new();
        if walk(start, oid, existing_depends_of, &mut path, &mut seen) {
            return Some(path);
        }
    }
    None
}

fn walk(
    at: &Oid,
    target: &Oid,
    edges: &dyn Fn(&Oid) -> Vec<Oid>,
    path: &mut Vec<Oid>,
    seen: &mut BTreeSet<Oid>,
) -> bool {
    if at == target {
        return true;
    }
    if !seen.insert(at.clone()) {
        return false;
    }
    for next in edges(at) {
        path.push(next.clone());
        if walk(&next, target, edges, path, seen) {
            return true;
        }
        path.pop();
    }
    false
}

#[cfg(test)]
mod tests;
