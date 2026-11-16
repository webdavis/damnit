use dam_domain::Permission::{PendingRead, TaskRead};

use super::*;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::{allowed, holder};

fn reader_of(paths: &[&str]) -> crate::access::Holder {
    let mut reader = holder(vec![allowed(&[TaskRead, PendingRead], Some(paths))]);
    reader.name = "reader".into();
    reader
}

fn a_set_in_work_and_home(s: &Scenario) {
    waiting(
        s,
        1,
        Change::Create {
            after: Object::Task(task_in(1, "a", "work")),
        },
    );
    waiting(
        s,
        2,
        Change::Create {
            after: Object::Task(task_in(2, "b", "home")),
        },
    );
}

fn named_by(s: &Scenario, reader: &crate::access::Holder) -> Result<Named, UseCaseError> {
    Dam {
        access: reader,
        ..s.dam()
    }
    .resolve_named(set().as_str())
}

#[test]
fn a_token_that_sees_none_or_part_of_a_set_does_not_find_it() {
    let s = Scenario::default();
    a_set_in_work_and_home(&s);
    for paths in [&["garden"][..], &["work"][..]] {
        let reader = reader_of(paths);
        assert_eq!(
            named_by(&s, &reader),
            Ok(Named::Object(set().oid().clone()))
        );
        let set = set();
        let prefix = &set.as_str()[..6];
        assert_eq!(
            Dam {
                access: &reader,
                ..s.dam()
            }
            .resolve_named(prefix),
            Err(UseCaseError::Refused(Refusal::NoWorkingObject(
                prefix.into()
            )))
        );
    }
}

#[test]
fn a_token_that_sees_all_of_a_set_and_its_author_find_it() {
    let s = Scenario::default();
    a_set_in_work_and_home(&s);
    assert_eq!(
        named_by(&s, &reader_of(&["work", "home"])),
        Ok(Named::Set(set()))
    );
    let author = holder(vec![]);
    assert_eq!(named_by(&s, &author), Ok(Named::Set(set())));
}
