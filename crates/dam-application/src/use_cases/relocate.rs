use dam_domain::{Oid, Path};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::move_into;

impl Dam<'_> {
    pub fn relocate(&self, oid: &Oid, to: &Path) -> Result<(), UseCaseError> {
        atomically(self.store, || {
            let object = load(self.store, oid)?;
            if object
                .base()
                .owned_folder()
                .is_some_and(|folder| to.is_within(&folder))
            {
                return Err(Refusal::MoveInsideItself(oid.clone()).into());
            }
            move_into(self.store, oid, to)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FaultyStore, Scenario, oid, task_in};

    fn mv(s: &Scenario, byte: u8, to: &str) -> Result<(), UseCaseError> {
        s.dam().relocate(&oid(byte), &Path::parse(to).unwrap())
    }

    #[test]
    fn moving_a_parent_carries_its_descendants() {
        let s = Scenario::default();
        s.filed(task_in(1, "plan", "a"))
            .filed(task_in(2, "step", "a/plan"))
            .filed(task_in(3, "detail", "a/plan/step"));
        mv(&s, 1, "b").unwrap();
        assert_eq!(s.path_of(&oid(1)), "b/");
        assert_eq!(s.path_of(&oid(2)), "b/plan/");
        assert_eq!(s.path_of(&oid(3)), "b/plan/step/");
    }

    #[test]
    fn moving_into_yourself_is_refused() {
        let s = Scenario::default();
        s.filed(task_in(1, "plan", "a"));
        let err = mv(&s, 1, "a/plan/c").unwrap_err();
        assert_eq!(
            err,
            UseCaseError::Refused(Refusal::MoveInsideItself(oid(1)))
        );
    }

    #[test]
    fn a_top_level_object_lands_exactly_at_the_destination() {
        let s = Scenario::default();
        s.filed(task_in(1, "x", ""));
        mv(&s, 1, "work").unwrap();
        assert_eq!(s.path_of(&oid(1)), "work/");
    }

    #[test]
    fn moving_a_top_level_object_leaves_the_rest_of_the_store_where_it_was() {
        let s = Scenario::default();
        s.filed(task_in(1, "a", ""))
            .filed(task_in(2, "b", ""))
            .filed(task_in(3, "note", "home"))
            .filed(task_in(4, "plan", "work"));
        mv(&s, 1, "work").unwrap();
        assert_eq!(s.path_of(&oid(1)), "work/");
        assert_eq!(s.path_of(&oid(2)), "");
        assert_eq!(s.path_of(&oid(3)), "home/");
        assert_eq!(s.path_of(&oid(4)), "work/");
    }

    #[test]
    fn moving_a_top_level_object_takes_its_children_and_nothing_else() {
        let s = Scenario::default();
        s.filed(task_in(1, "a", ""))
            .filed(task_in(2, "child", "a"))
            .filed(task_in(3, "b", ""))
            .filed(task_in(4, "other", "x"));
        mv(&s, 1, "work").unwrap();
        assert_eq!(s.path_of(&oid(1)), "work/");
        assert_eq!(s.path_of(&oid(2)), "work/a/");
        assert_eq!(s.path_of(&oid(3)), "");
        assert_eq!(s.path_of(&oid(4)), "x/");
    }

    #[test]
    fn moving_to_the_root_puts_the_object_and_its_children_at_the_top_level() {
        let s = Scenario::default();
        s.filed(task_in(1, "plan", "work"))
            .filed(task_in(2, "step", "work/plan"));
        mv(&s, 1, "").unwrap();
        assert_eq!(s.path_of(&oid(1)), "");
        assert_eq!(s.path_of(&oid(2)), "plan/");
    }

    #[test]
    fn mv_that_fails_between_parent_and_child_moves_neither() {
        let s = Scenario::default();
        s.filed(task_in(1, "p", "a"))
            .filed(task_in(2, "c", "a/p"))
            .filed(task_in(3, "d", "a/p"));
        let store = FaultyStore::new(s.store);
        let dam = s.fixture.dam(&store);
        store.assert_unchanged_when_any_write_fails(|| {
            dam.relocate(&oid(1), &Path::parse("b").unwrap())
        });
    }
}
