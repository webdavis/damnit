use std::collections::BTreeSet;

use crate::Oid;

/// Why `done` is refused. Dependencies are listed before children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Blocker {
    OpenDependency(Oid),
    OpenChild(Oid),
}

/// How far a caller will go to complete a blocked task. `With` carries the
/// answers to what happens to the blockers, so a forced completion can never
/// be asked to dispose of them without saying how.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Force {
    No,
    Yes,
    With(Dispositions),
}

/// What happens to a forced parent's open children and open dependencies.
/// Both default to keeping what is there, which is the least a completion can
/// disturb and what a caller that names only one of them gets for the other.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dispositions {
    pub children: ChildDisposition,
    pub dependencies: DependencyDisposition,
}

/// What to do with open children when a parent is forced done.
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

/// Pure: the caller supplies which dependencies and children are still open.
pub fn blockers(open_dependencies: &[Oid], open_children: &[Oid]) -> Vec<Blocker> {
    open_dependencies
        .iter()
        .cloned()
        .map(Blocker::OpenDependency)
        .chain(open_children.iter().cloned().map(Blocker::OpenChild))
        .collect()
}

/// The path from one of `depends` back to `oid`, if adding `depends` to `oid`
/// would close a cycle. `edges` answers what an object already depends on.
pub fn cycle_in(depends: &[Oid], oid: &Oid, edges: &dyn Fn(&Oid) -> Vec<Oid>) -> Option<Vec<Oid>> {
    for start in depends {
        let mut path = vec![start.clone()];
        let mut seen = BTreeSet::new();
        if walk(start, oid, edges, &mut path, &mut seen) {
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
mod tests {
    use super::*;
    use crate::Oid;
    use std::collections::BTreeMap;

    fn oid(byte: u8) -> Oid {
        Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
    }

    #[test]
    fn no_open_blockers_means_none() {
        assert!(blockers(&[], &[]).is_empty());
    }

    #[test]
    fn open_dependencies_come_before_open_children() {
        let found = blockers(&[oid(1)], &[oid(2), oid(3)]);
        assert_eq!(
            found,
            vec![
                Blocker::OpenDependency(oid(1)),
                Blocker::OpenChild(oid(2)),
                Blocker::OpenChild(oid(3))
            ]
        );
    }

    #[test]
    fn a_direct_cycle_is_found() {
        // b depends on a; adding a -> b closes the loop
        let graph: BTreeMap<Oid, Vec<Oid>> = [(oid(2), vec![oid(1)])].into();
        let edges = |o: &Oid| graph.get(o).cloned().unwrap_or_default();
        assert_eq!(
            cycle_in(&[oid(2)], &oid(1), &edges),
            Some(vec![oid(2), oid(1)])
        );
    }

    #[test]
    fn a_transitive_cycle_is_found() {
        // c -> b -> a; adding a -> c closes it
        let graph: BTreeMap<Oid, Vec<Oid>> =
            [(oid(3), vec![oid(2)]), (oid(2), vec![oid(1)])].into();
        let edges = |o: &Oid| graph.get(o).cloned().unwrap_or_default();
        assert_eq!(
            cycle_in(&[oid(3)], &oid(1), &edges),
            Some(vec![oid(3), oid(2), oid(1)])
        );
    }

    #[test]
    fn no_cycle_returns_none() {
        let graph: BTreeMap<Oid, Vec<Oid>> = [(oid(2), vec![oid(3)])].into();
        let edges = |o: &Oid| graph.get(o).cloned().unwrap_or_default();
        assert_eq!(cycle_in(&[oid(2)], &oid(1), &edges), None);
    }

    #[test]
    fn depending_on_yourself_is_a_cycle() {
        let edges = |_: &Oid| Vec::new();
        assert_eq!(cycle_in(&[oid(1)], &oid(1), &edges), Some(vec![oid(1)]));
    }
}
