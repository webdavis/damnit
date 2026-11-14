use dam_domain::PathError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub file: String,
    pub table: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expected {
    Text,
    TrueOrFalse,
    ListOfTexts,
    ListOfTables,
    Table,
    TableOfNamedEntries,
    TextOrFunction,
    DateTime,
    TableOfWholeNumbers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Misread {
    UnknownKey {
        key: String,
        at: Place,
    },
    UnknownRemoteKey {
        key: String,
        at: Place,
        takes: Vec<String>,
    },
    WrongType {
        key: String,
        at: Place,
        expected: Expected,
    },
    Missing {
        key: String,
        at: Place,
    },
    NotAPath {
        text: String,
        at: Place,
        why: PathError,
    },
    NotAHelperUrl {
        url: String,
        at: Place,
    },
    NotADuration {
        key: String,
        text: String,
        at: Place,
    },
}
