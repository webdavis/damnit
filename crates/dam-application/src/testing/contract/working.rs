//! The working layer and the stage: what a store must do with objects the
//! operator is editing and with the changes they have staged.

use crate::ports::{ObjectRepository, StageRepository};
use dam_domain::{Change, Object, Oid, Op, Path};

use super::{oid, task};

/// Reads and writes of the working layer: replacement, deletion, the tree
/// query, the dependency query, and the committed view that `put` never moves.
pub fn object_repository_contract(objects: &dyn ObjectRepository) {
    assert_eq!(objects.get(&oid(1)).unwrap(), None, "absent reads as none");
    assert!(
        objects.all().unwrap().is_empty(),
        "an empty store lists none"
    );

    let parent = task(1, "parent", "work");
    objects.put(&parent).unwrap();
    assert_eq!(objects.get(&oid(1)).unwrap(), Some(parent.clone()));

    let renamed = task(1, "renamed", "work");
    objects.put(&renamed).unwrap();
    assert_eq!(
        objects.get(&oid(1)).unwrap(),
        Some(renamed),
        "a second put replaces rather than duplicates"
    );
    assert_eq!(objects.all().unwrap().len(), 1);

    objects.put(&task(2, "child", "work/parent")).unwrap();
    objects
        .put(&task(4, "grandchild", "work/parent/deep"))
        .unwrap();
    objects.put(&task(5, "elsewhere", "workshop")).unwrap();
    assert_eq!(
        oids_of(objects.children_of(&Path::parse("work").unwrap()).unwrap()),
        vec![oid(2)],
        "children_of answers with the immediate children alone: not the node \
         itself, not a grandchild, and not a path that merely shares a prefix"
    );
    assert_eq!(
        oids_of(
            objects
                .children_of(&Path::parse("work/parent").unwrap())
                .unwrap()
        ),
        vec![oid(4)]
    );

    let mut dependent = task(3, "dependent", "work");
    dependent.base_mut().depends.push(oid(1));
    objects.put(&dependent).unwrap();
    assert_eq!(objects.dependents_of(&oid(1)).unwrap(), vec![oid(3)]);
    assert!(objects.dependents_of(&oid(2)).unwrap().is_empty());

    assert_eq!(
        objects.committed(&oid(1)).unwrap(),
        None,
        "putting into working never touches the committed view"
    );

    let before = objects.all().unwrap().len();
    objects.delete(&oid(3)).unwrap();
    assert_eq!(objects.get(&oid(3)).unwrap(), None);
    objects.delete(&oid(3)).unwrap();
    assert_eq!(
        objects.all().unwrap().len(),
        before - 1,
        "deleting what is already gone is not an error and removes nothing else"
    );
}

pub(super) fn oids_of(objects: Vec<Object>) -> Vec<Oid> {
    let mut out: Vec<Oid> = objects.iter().map(|o| o.oid().clone()).collect();
    out.sort();
    out
}

/// The stage holds at most one change per oid, coalescing a second one onto
/// the first, and a create followed by a delete leaves nothing staged.
pub fn stage_repository_contract(stage: &dyn StageRepository) {
    assert!(stage.staged().unwrap().is_empty());

    let created = task(1, "created", "");
    stage
        .stage(Change {
            oid: oid(1),
            op: Op::Create,
            before: None,
            after: Some(created.clone()),
        })
        .unwrap();
    assert_eq!(stage.staged().unwrap().len(), 1);

    let edited = task(1, "edited", "");
    stage
        .stage(Change {
            oid: oid(1),
            op: Op::Update,
            before: Some(created.clone()),
            after: Some(edited.clone()),
        })
        .unwrap();
    let staged = stage.staged().unwrap();
    assert_eq!(staged.len(), 1, "one oid stages once");
    assert_eq!(staged[0].op, Op::Create, "a create absorbs a later update");
    assert_eq!(staged[0].after, Some(edited.clone()));

    stage
        .stage(Change {
            oid: oid(1),
            op: Op::Delete,
            before: Some(edited),
            after: None,
        })
        .unwrap();
    assert!(
        stage.staged().unwrap().is_empty(),
        "a create then a delete stages nothing"
    );

    stage
        .stage(Change {
            oid: oid(2),
            op: Op::Create,
            before: None,
            after: Some(task(2, "other", "")),
        })
        .unwrap();
    stage.unstage(&oid(1)).unwrap();
    assert_eq!(
        stage.staged().unwrap().len(),
        1,
        "unstaging an oid that is not staged leaves the others alone"
    );
    stage.unstage_all().unwrap();
    assert!(stage.staged().unwrap().is_empty());
}
