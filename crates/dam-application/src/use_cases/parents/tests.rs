mod kept;
mod pulled;

use dam_domain::{
    CallerId, ChildDisposition, DependencyDisposition, Dispositions, Force, Oid, Path, Priority,
    Tracked,
};

use crate::ports::ObjectRepository;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::edit::EditFields;
use crate::use_cases::new_object::NewTask;

fn parent_of(s: &Scenario, oid: &Oid) -> Option<Oid> {
    s.store.get(oid).unwrap().unwrap().base().parent.clone()
}

fn errands() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(1, "errands", "Home"));
    s
}

fn new_task(path: &str, id: Option<&str>) -> NewTask {
    NewTask {
        parent: None,
        subject: "milk".into(),
        path: Path::parse(path).unwrap(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: Default::default(),
        body: String::new(),
        id: id.map(|id| CallerId::parse(id).unwrap()),
    }
}

#[test]
fn a_new_task_takes_the_object_that_holds_its_path_as_parent() {
    let s = errands();
    let milk = s
        .dam()
        .new_task(new_task("Home/errands/later", None))
        .unwrap()
        .written();
    assert_eq!(parent_of(&s, milk.oid()), Some(oid(1)));
}

#[test]
fn restating_a_task_by_id_into_a_folder_takes_its_holder_as_parent() {
    let s = errands();
    let dam = s.dam();
    let milk = dam.new_task(new_task("", Some("milk"))).unwrap().written();
    assert_eq!(parent_of(&s, milk.oid()), None);
    dam.new_task(new_task("Home/errands", Some("milk")))
        .unwrap();
    assert_eq!(parent_of(&s, milk.oid()), Some(oid(1)));
}

#[test]
fn a_move_sets_the_parent_of_the_object_and_of_its_contents() {
    let s = errands();
    s.filed(task_in(2, "milk", ""))
        .filed(task_in(3, "oat", "milk"));
    s.dam()
        .relocate(&oid(2), Some(&Path::parse("Home/errands").unwrap()), None)
        .unwrap();
    assert_eq!(s.path_of(&oid(3)), "Home/errands/milk/");
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
    assert_eq!(parent_of(&s, &oid(3)), Some(oid(2)));
}

#[test]
fn a_rename_keeps_the_contents_parented_by_the_renamed_object() {
    let s = errands();
    s.filed(task_in(2, "milk", "Home/errands"));
    let rename = EditFields {
        subject: Some("chores".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(1), rename).unwrap();
    assert_eq!(s.path_of(&oid(2)), "Home/chores/");
    assert_eq!(parent_of(&s, &oid(1)), Some(oid(5)));
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
}

#[test]
fn removing_an_object_gives_its_contents_the_next_holder_up() {
    let s = errands();
    s.filed(task_in(2, "milk", "Home/errands"));
    s.dam().remove(&oid(1)).unwrap();
    assert_eq!(s.path_of(&oid(2)), "Home/");
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(5)));
}

#[test]
fn done_with_children_into_a_group_parents_the_group_and_the_children() {
    let s = errands();
    s.filed(task_in(2, "milk", "Home/errands"));
    let into = Force::With(Dispositions {
        children: ChildDisposition::Into("leftovers".into()),
        dependencies: DependencyDisposition::Keep,
    });
    s.dam().complete(&oid(1), into, Tracked::No).unwrap();
    assert_eq!(s.path_of(&oid(2)), "Home/leftovers/");
    let group = parent_of(&s, &oid(2)).expect("the group holds the children");
    assert_eq!(parent_of(&s, &group), Some(oid(5)));
}
