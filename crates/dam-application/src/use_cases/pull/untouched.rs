pub(super) struct AtThePull {
    pub(super) uncommitted: bool,
    pub(super) changed_in_the_app: bool,
    pub(super) the_pull_would_write: bool,
}

#[derive(Debug, PartialEq)]
pub(super) enum Untouched {
    LeftAlone,
    Conflict,
}

impl Untouched {
    pub(super) fn of(facts: AtThePull) -> Option<Untouched> {
        let AtThePull {
            uncommitted,
            changed_in_the_app,
            the_pull_would_write,
        } = facts;
        match (uncommitted, changed_in_the_app, the_pull_would_write) {
            (false, _, _) | (true, true, false) => None,
            (true, false, _) => Some(Untouched::LeftAlone),
            (true, true, true) => Some(Untouched::Conflict),
        }
    }
}
