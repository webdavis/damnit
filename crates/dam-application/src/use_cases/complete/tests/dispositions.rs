use super::super::*;
use super::{at, filed, path_of};
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Scenario, oid};

fn with(children: ChildDisposition) -> Force {
    Force::With(Dispositions {
        children,
        dependencies: DependencyDisposition::Keep,
    })
}

fn plan_with_a_step() -> (Scenario, Oid, Oid) {
    let s = Scenario::default().drawing_from(9);
    let plan = filed(&s, at(1, "plan", "work"));
    let step = filed(&s, at(2, "step", "work/plan"));
    (s, plan, step)
}

fn milk_blocked_by_an_open_dependency() -> (Scenario, Oid) {
    let s = Scenario::default().drawing_from(9);
    let mut milk = at(1, "milk", "");
    milk.base.depends.push(oid(3));
    let id = filed(&s, milk);
    filed(&s, at(3, "blocker", ""));
    (s, id)
}

#[test]
fn interactive_up_moves_children_beside_the_parent() {
    let (s, plan, step) = plan_with_a_step();
    s.dam()
        .complete(&plan, with(ChildDisposition::Up), Tracked::No)
        .unwrap();
    assert_eq!(path_of(&s, &step), "work/");
}

#[test]
fn interactive_into_groups_children_under_a_new_task_beside_the_parent() {
    let (s, plan, step) = plan_with_a_step();
    s.dam()
        .complete(
            &plan,
            with(ChildDisposition::Into("leftovers".into())),
            Tracked::No,
        )
        .unwrap();
    let group = s.store.get(&oid(9)).unwrap().unwrap();
    assert_eq!(group.base().subject, "leftovers");
    assert_eq!(group.base().path.as_str(), "work/");
    assert_eq!(path_of(&s, &step), "work/leftovers/");
}

#[test]
fn interactive_up_carries_a_grandchild_along() {
    let (s, plan, step) = plan_with_a_step();
    let detail = filed(&s, at(4, "detail", "work/plan/step"));
    s.dam()
        .complete(&plan, with(ChildDisposition::Up), Tracked::No)
        .unwrap();
    assert_eq!(path_of(&s, &step), "work/");
    assert_eq!(path_of(&s, &detail), "work/step/");
}

#[test]
fn interactive_into_carries_a_grandchild_along() {
    let (s, plan, step) = plan_with_a_step();
    let detail = filed(&s, at(4, "detail", "work/plan/step"));
    s.dam()
        .complete(
            &plan,
            with(ChildDisposition::Into("later".into())),
            Tracked::No,
        )
        .unwrap();
    assert_eq!(path_of(&s, &step), "work/later/");
    assert_eq!(path_of(&s, &detail), "work/later/step/");
}

#[test]
fn interactive_into_with_no_open_children_creates_no_group() {
    let (s, id) = milk_blocked_by_an_open_dependency();
    s.dam()
        .complete(
            &id,
            with(ChildDisposition::Into("later".into())),
            Tracked::No,
        )
        .unwrap();
    assert!(s.store.get(&oid(9)).unwrap().is_none());
}

#[test]
fn interactive_drop_removes_open_dependencies() {
    let (s, id) = milk_blocked_by_an_open_dependency();
    let d = Dispositions {
        children: ChildDisposition::Keep,
        dependencies: DependencyDisposition::Drop,
    };
    s.dam().complete(&id, Force::With(d), Tracked::No).unwrap();
    let t = s.store.get(&id).unwrap().unwrap();
    assert!(t.base().depends.is_empty() && t.as_task().unwrap().completion.is_done());
}

#[test]
fn done_up_that_fails_between_two_children_moves_neither() {
    let (s, plan, _) = plan_with_a_step();
    filed(&s, at(3, "other step", "work/plan"));
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| {
        dam.complete(&plan, with(ChildDisposition::Up), Tracked::No)
            .map(drop)
    });
}

#[test]
fn done_into_that_fails_after_creating_the_group_leaves_no_group_behind() {
    let s = Scenario::default().drawing_from(9);
    let mut plan = at(1, "plan", "work");
    plan.base.depends.push(oid(3));
    let plan = filed(&s, plan);
    filed(&s, at(2, "step", "work/plan"));
    filed(&s, at(3, "blocker", ""));
    let d = Dispositions {
        children: ChildDisposition::Into("leftovers".into()),
        dependencies: DependencyDisposition::Drop,
    };
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| {
        dam.complete(&plan, Force::With(d.clone()), Tracked::No)
            .map(drop)
    });
}
