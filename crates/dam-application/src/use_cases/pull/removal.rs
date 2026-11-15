use dam_domain::Object;

pub(super) struct AtTheRemoval {
    pub(super) dam_sent_a_delete: bool,
    pub(super) committed: Option<Object>,
    pub(super) in_the_working_copy: bool,
}

#[derive(Debug, PartialEq)]
pub(super) struct Removal {
    pub(super) kept_as_owing_nothing: Option<Object>,
    pub(super) noticed: bool,
}

impl Removal {
    pub(super) fn of(facts: AtTheRemoval) -> Removal {
        let removed_in_the_app = !facts.dam_sent_a_delete;
        Removal {
            kept_as_owing_nothing: facts.committed.filter(|_| removed_in_the_app),
            noticed: removed_in_the_app && facts.in_the_working_copy,
        }
    }
}
