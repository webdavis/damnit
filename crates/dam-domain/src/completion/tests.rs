use super::*;
use crate::Oid;
use std::collections::BTreeMap;

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

#[test]
fn no_open_dependency_and_no_open_child_means_nothing_blocks() {
    assert!(Blockers::default().is_empty());
    let one_child = Blockers {
        children: vec![oid(2)],
        ..Blockers::default()
    };
    assert!(!one_child.is_empty());
    let one_dependency = Blockers {
        dependencies: vec![oid(1)],
        ..Blockers::default()
    };
    assert!(!one_dependency.is_empty());
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
