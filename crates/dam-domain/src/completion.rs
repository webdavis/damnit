use std::collections::BTreeSet;

use crate::Oid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Blocker {
    OpenDependency(Oid),
    OpenChild(Oid),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Force {
    No,
    Yes,
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

pub fn blockers(open_dependencies: &[Oid], open_children: &[Oid]) -> Vec<Blocker> {
    open_dependencies
        .iter()
        .cloned()
        .map(Blocker::OpenDependency)
        .chain(open_children.iter().cloned().map(Blocker::OpenChild))
        .collect()
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
        let (a, b) = (oid(1), oid(2));
        let b_depends_on_a: BTreeMap<Oid, Vec<Oid>> = [(b.clone(), vec![a.clone()])].into();
        let edges = |o: &Oid| b_depends_on_a.get(o).cloned().unwrap_or_default();
        let a_now_depending_on_b = cycle_in(std::slice::from_ref(&b), &a, &edges);
        assert_eq!(a_now_depending_on_b, Some(vec![b, a]));
    }

    #[test]
    fn a_transitive_cycle_is_found() {
        let (a, b, c) = (oid(1), oid(2), oid(3));
        let c_depends_on_b_depends_on_a: BTreeMap<Oid, Vec<Oid>> =
            [(c.clone(), vec![b.clone()]), (b.clone(), vec![a.clone()])].into();
        let edges = |o: &Oid| {
            c_depends_on_b_depends_on_a
                .get(o)
                .cloned()
                .unwrap_or_default()
        };
        let a_now_depending_on_c = cycle_in(std::slice::from_ref(&c), &a, &edges);
        assert_eq!(a_now_depending_on_c, Some(vec![c, b, a]));
    }

    #[test]
    fn no_cycle_returns_none() {
        let graph: BTreeMap<Oid, Vec<Oid>> = [(oid(2), vec![oid(3)])].into();
        let edges = |o: &Oid| graph.get(o).cloned().unwrap_or_default();
        assert_eq!(cycle_in(&[oid(2)], &oid(1), &edges), None);
    }

    #[test]
    fn dispositions_left_unnamed_keep_children_and_dependencies_where_they_are() {
        let only_children_named = Dispositions {
            children: ChildDisposition::Up,
            ..Dispositions::default()
        };
        assert_eq!(
            only_children_named.dependencies,
            DependencyDisposition::Keep
        );
        assert_eq!(Dispositions::default().children, ChildDisposition::Keep);
    }

    #[test]
    fn depending_on_yourself_is_a_cycle() {
        let edges = |_: &Oid| Vec::new();
        assert_eq!(cycle_in(&[oid(1)], &oid(1), &edges), Some(vec![oid(1)]));
    }
}
