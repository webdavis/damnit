#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditedTable {
    Remotes,
    Tokens,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uneditable {
    NotEditable,
    ReturnsBuiltInCode,
    TableBuiltInCode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditFault {
    WouldNotHoldTheEntry,
    WouldStillHoldTheEntry,
    WouldNotKeepTheOthers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ByHand {
    AddRemote { entry: String },
    AddToken { name: String },
    RemoveToken { name: String },
}
